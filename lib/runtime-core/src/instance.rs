use crate::{
    backend::Token,
    backing::{ImportBacking, LocalBacking},
    error::{CallError, CallResult, ResolveError, ResolveResult, Result},
    export::{Context, Export, ExportIter, FuncPointer},
    global::Global,
    import::ImportObject,
    memory::Memory,
    module::{Module, ModuleInner, ResourceIndex},
    sig_registry::SigRegistry,
    table::Table,
    typed_func::{Func, Wasm, WasmTypeList},
    types::{FuncIndex, FuncSig, GlobalIndex, LocalOrImport, MemoryIndex, TableIndex, Value},
    vm,
};
use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::{self, ManuallyDrop},
    pin::Pin,
    ptr::NonNull,
    sync::Arc,
};

pub(crate) struct InstanceInner<Data> {
    #[allow(dead_code)]
    pub(crate) backing: LocalBacking,
    import_backing: ImportBacking,
    pub(crate) ctx: UnsafeCell<vm::Ctx<Data>>,
}

// impl Drop for InstanceInner {
//     fn drop(&mut self) {
//         // Drop the ctx.
//         unsafe { Box::from_raw(self.ctx) };
//     }
// }

/// An instantiated WebAssembly module.
///
/// An `Instance` represents a WebAssembly module that
/// has been instantiated with an [`ImportObject`] and is
/// ready to be called.
///
/// [`ImportObject`]: struct.ImportObject.html
pub struct Instance<'imports, Data = ()> {
    module: Arc<ModuleInner>,
    inner: Pin<Box<InstanceInner<Data>>>,
    _phantom: PhantomData<&'imports ()>,
}

impl<'imports, Data> Instance<'imports, Data> {
    pub(crate) fn new(
        module: Arc<ModuleInner>,
        import_object: &'imports ImportObject<Data>,
    ) -> Result<Instance<'imports, Data>> {
        // We need the backing and import_backing to create a vm::Ctx, but we need
        // a vm::Ctx to create a backing and an import_backing. The solution is to create an
        // uninitialized InstanceInner on the heap and then initialize it in-place.
        let inner: Pin<Box<InstanceInner<Data>>> = unsafe {
            // The InstanceInner is wrapped in a ManuallyDrop to ensure that if we prematurely
            // exit from this function, we don't attempt to free uninitialized memory.
            let mut inner: Box<ManuallyDrop<InstanceInner<Data>>> =
                Box::new(ManuallyDrop::new(mem::uninitialized()));
            let ctx_ptr = inner.ctx.get();

            let mut import_backing = ImportBacking::new(&module, import_object, ctx_ptr as *mut _)?;
            let mut backing = LocalBacking::new(&module, &import_backing, ctx_ptr as *mut _);

            let real_ctx = vm::Ctx::new(
                &mut backing,
                &mut import_backing,
                &module,
                import_object.create_state(),
            );

            // Write into the InstanceInner without dropping the uninitilized fields.
            (&mut inner.backing as *mut LocalBacking).write(backing);
            (&mut inner.import_backing as *mut ImportBacking).write(import_backing);
            (&mut inner.ctx as *mut UnsafeCell<vm::Ctx<Data>>).write(UnsafeCell::new(real_ctx));

            let pinned: Pin<_> = inner.into();
            // Turn this into a normal box (without the ManuallyDrop).
            mem::transmute(pinned)
        };

        let instance = Instance {
            module,
            inner,
            _phantom: PhantomData,
        };

        if let Some(start_index) = instance.module.info.start_func {
            instance.call_with_index(start_index, &[])?;
        }

        Ok(instance)
    }

    /// Through generic magic and the awe-inspiring power of traits, we bring you...
    ///
    /// # "Func"
    ///
    /// A [`Func`] allows you to call functions exported from wasm with
    /// near zero overhead.
    ///
    /// [`Func`]: struct.Func.html
    /// # Usage:
    ///
    /// ```
    /// # use wasmer_runtime_core::{Func, Instance, error::ResolveResult};
    /// # fn typed_func(instance: Instance) -> ResolveResult<()> {
    /// let func: Func<(i32, i32)> = instance.func("foo")?;
    ///
    /// func.call(42, 43);
    /// # Ok(())
    /// # }
    /// ```
    pub fn func<Args, Rets>(&self, name: &str) -> ResolveResult<Func<Args, Rets, Data, Wasm>>
    where
        Args: WasmTypeList,
        Rets: WasmTypeList,
    {
        let export_index =
            self.module
                .info
                .exports
                .get(name)
                .ok_or_else(|| ResolveError::ExportNotFound {
                    name: name.to_string(),
                })?;

        if let ResourceIndex::Func(func_index) = export_index {
            let sig_index = *self
                .module
                .info
                .func_assoc
                .get(*func_index)
                .expect("broken invariant, incorrect func index");
            let signature =
                SigRegistry.lookup_signature_ref(&self.module.info.signatures[sig_index]);

            if signature.params() != Args::types() || signature.returns() != Rets::types() {
                Err(ResolveError::Signature {
                    expected: (*signature).clone(),
                    found: Args::types().to_vec(),
                })?;
            }

            let ctx = match func_index.local_or_import(&self.module.info) {
                LocalOrImport::Local(_) => self.inner.ctx.get(),
                LocalOrImport::Import(imported_func_index) => {
                    self.inner.import_backing.vm_functions[imported_func_index].ctx as *mut _
                }
            };

            let func_wasm_inner = self
                .module
                .protected_caller
                .get_wasm_trampoline(&self.module, sig_index)
                .unwrap();

            let func_ptr = match func_index.local_or_import(&self.module.info) {
                LocalOrImport::Local(local_func_index) => self
                    .module
                    .func_resolver
                    .get(&self.module, local_func_index)
                    .unwrap(),
                LocalOrImport::Import(import_func_index) => NonNull::new(
                    self.inner.import_backing.vm_functions[import_func_index].func as *mut _,
                )
                .unwrap(),
            };

            let typed_func: Func<Args, Rets, Data, Wasm> =
                unsafe { Func::from_raw_parts(func_wasm_inner, func_ptr, ctx) };

            Ok(typed_func)
        } else {
            Err(ResolveError::ExportWrongType {
                name: name.to_string(),
            }
            .into())
        }
    }

    /// This returns the representation of a function that can be called
    /// safely.
    ///
    /// # Usage:
    /// ```
    /// # use wasmer_runtime_core::Instance;
    /// # use wasmer_runtime_core::error::CallResult;
    /// # fn call_foo(instance: &mut Instance) -> CallResult<()> {
    /// instance
    ///     .dyn_func("foo")?
    ///     .call(&[])?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn dyn_func(&self, name: &str) -> ResolveResult<DynFunc<Data>> {
        let export_index =
            self.module
                .info
                .exports
                .get(name)
                .ok_or_else(|| ResolveError::ExportNotFound {
                    name: name.to_string(),
                })?;

        if let ResourceIndex::Func(func_index) = export_index {
            let sig_index = *self
                .module
                .info
                .func_assoc
                .get(*func_index)
                .expect("broken invariant, incorrect func index");
            let signature =
                SigRegistry.lookup_signature_ref(&self.module.info.signatures[sig_index]);

            Ok(DynFunc {
                signature,
                module: &self.module,
                instance_inner: &*self.inner,
                func_index: *func_index,
            })
        } else {
            Err(ResolveError::ExportWrongType {
                name: name.to_string(),
            }
            .into())
        }
    }

    /// Call an exported webassembly function given the export name.
    /// Pass arguments by wrapping each one in the [`Value`] enum.
    /// The returned values are also each wrapped in a [`Value`].
    ///
    /// [`Value`]: enum.Value.html
    ///
    /// # Note:
    /// This returns `CallResult<Vec<Value>>` in order to support
    /// the future multi-value returns webassembly feature.
    ///
    /// # Usage:
    /// ```
    /// # use wasmer_runtime_core::types::Value;
    /// # use wasmer_runtime_core::error::Result;
    /// # use wasmer_runtime_core::Instance;
    /// # fn call_foo(instance: &mut Instance) -> Result<()> {
    /// // ...
    /// let results = instance.call("foo", &[Value::I32(42)])?;
    /// // ...
    /// # Ok(())
    /// # }
    /// ```
    pub fn call(&self, name: &str, args: &[Value]) -> CallResult<Vec<Value>> {
        let export_index =
            self.module
                .info
                .exports
                .get(name)
                .ok_or_else(|| ResolveError::ExportNotFound {
                    name: name.to_string(),
                })?;

        let func_index = if let ResourceIndex::Func(func_index) = export_index {
            *func_index
        } else {
            return Err(CallError::Resolve(ResolveError::ExportWrongType {
                name: name.to_string(),
            })
            .into());
        };

        self.call_with_index(func_index, args)
    }

    /// Returns an immutable reference to the
    /// [`Ctx`] used by this Instance.
    ///
    /// [`Ctx`]: struct.Ctx.html
    pub fn context(&self) -> &vm::Ctx<Data> {
        unsafe { &*self.inner.ctx.get() }
    }

    /// Returns a mutable reference to the
    /// [`Ctx`] used by this Instance.
    ///
    /// [`Ctx`]: struct.Ctx.html
    pub fn context_mut(&mut self) -> &mut vm::Ctx<Data> {
        unsafe { &mut *self.inner.ctx.get() }
    }

    /// Returns an iterator over all of the items
    /// exported from this instance.
    pub fn exports(&self) -> ExportIter<Data> {
        ExportIter::new(&self.module, &self.inner)
    }

    pub fn export(&self, name: &str) -> Option<Export> {
        let export_index = self.module.info.exports.get(name)?;

        Some(
            self.inner
                .get_export_from_index(&*self.module, export_index),
        )
    }

    /// The module used to instantiate this Instance.
    pub fn module(&self) -> Module {
        Module::new(Arc::clone(&self.module))
    }
}

impl<'imports, Data> Instance<'imports, Data> {
    fn call_with_index(&self, func_index: FuncIndex, args: &[Value]) -> CallResult<Vec<Value>> {
        let sig_index = *self
            .module
            .info
            .func_assoc
            .get(func_index)
            .expect("broken invariant, incorrect func index");
        let signature = &self.module.info.signatures[sig_index];

        if !signature.check_param_value_types(args) {
            Err(ResolveError::Signature {
                expected: signature.clone(),
                found: args.iter().map(|val| val.ty()).collect(),
            })?
        }

        let ctx = match func_index.local_or_import(&self.module.info) {
            LocalOrImport::Local(_) => self.inner.ctx.get() as *mut vm::Ctx,
            LocalOrImport::Import(imported_func_index) => {
                self.inner.import_backing.vm_functions[imported_func_index].ctx
            }
        };

        let token = Token::generate();

        let returns = self.module.protected_caller.call(
            &self.module,
            func_index,
            args,
            &self.inner.import_backing,
            ctx,
            token,
        )?;

        Ok(returns)
    }
}

impl<Data> InstanceInner<Data> {
    pub(crate) fn get_export_from_index(
        &self,
        module: &ModuleInner,
        export_index: &ResourceIndex,
    ) -> Export {
        match export_index {
            ResourceIndex::Func(func_index) => {
                let (func, ctx, signature) = self.get_func_from_index(module, *func_index);

                Export::Function {
                    func,
                    ctx: match ctx {
                        Context::Internal => Context::External(self.ctx.get() as *mut vm::Ctx),
                        ctx @ Context::External(_) => ctx,
                    },
                    signature,
                    _marker: PhantomData,
                }
            }
            ResourceIndex::Memory(memory_index) => {
                let memory = self.get_memory_from_index(module, *memory_index);
                Export::Memory(memory)
            }
            ResourceIndex::Global(global_index) => {
                let global = self.get_global_from_index(module, *global_index);
                Export::Global(global)
            }
            ResourceIndex::Table(table_index) => {
                let table = self.get_table_from_index(module, *table_index);
                Export::Table(table)
            }
        }
    }

    fn get_func_from_index(
        &self,
        module: &ModuleInner,
        func_index: FuncIndex,
    ) -> (FuncPointer, Context, Arc<FuncSig>) {
        let sig_index = *module
            .info
            .func_assoc
            .get(func_index)
            .expect("broken invariant, incorrect func index");

        let (func_ptr, ctx) = match func_index.local_or_import(&module.info) {
            LocalOrImport::Local(local_func_index) => (
                module
                    .func_resolver
                    .get(&module, local_func_index)
                    .expect("broken invariant, func resolver not synced with module.exports")
                    .cast()
                    .as_ptr() as *const _,
                Context::Internal,
            ),
            LocalOrImport::Import(imported_func_index) => {
                let imported_func = &self.import_backing.vm_functions[imported_func_index];
                (
                    imported_func.func as *const _,
                    Context::External(imported_func.ctx),
                )
            }
        };

        let signature = SigRegistry.lookup_signature_ref(&module.info.signatures[sig_index]);
        // let signature = &module.info.signatures[sig_index];

        (unsafe { FuncPointer::new(func_ptr) }, ctx, signature)
    }

    fn get_memory_from_index(&self, module: &ModuleInner, mem_index: MemoryIndex) -> Memory {
        match mem_index.local_or_import(&module.info) {
            LocalOrImport::Local(local_mem_index) => self.backing.memories[local_mem_index].clone(),
            LocalOrImport::Import(imported_mem_index) => {
                self.import_backing.memories[imported_mem_index].clone()
            }
        }
    }

    fn get_global_from_index(&self, module: &ModuleInner, global_index: GlobalIndex) -> Global {
        match global_index.local_or_import(&module.info) {
            LocalOrImport::Local(local_global_index) => {
                self.backing.globals[local_global_index].clone()
            }
            LocalOrImport::Import(import_global_index) => {
                self.import_backing.globals[import_global_index].clone()
            }
        }
    }

    fn get_table_from_index(&self, module: &ModuleInner, table_index: TableIndex) -> Table {
        match table_index.local_or_import(&module.info) {
            LocalOrImport::Local(local_table_index) => {
                self.backing.tables[local_table_index].clone()
            }
            LocalOrImport::Import(imported_table_index) => {
                self.import_backing.tables[imported_table_index].clone()
            }
        }
    }
}

/// A representation of an exported WebAssembly function.
pub struct DynFunc<'a, Data = ()> {
    pub(crate) signature: Arc<FuncSig>,
    module: &'a ModuleInner,
    pub(crate) instance_inner: &'a InstanceInner<Data>,
    func_index: FuncIndex,
}

impl<'a, Data> DynFunc<'a, Data> {
    /// Call an exported webassembly function safely.
    ///
    /// Pass arguments by wrapping each one in the [`Value`] enum.
    /// The returned values are also each wrapped in a [`Value`].
    ///
    /// [`Value`]: enum.Value.html
    ///
    /// # Note:
    /// This returns `CallResult<Vec<Value>>` in order to support
    /// the future multi-value returns webassembly feature.
    ///
    /// # Usage:
    /// ```
    /// # use wasmer_runtime_core::Instance;
    /// # use wasmer_runtime_core::error::CallResult;
    /// # fn call_foo(instance: &mut Instance) -> CallResult<()> {
    /// instance
    ///     .dyn_func("foo")?
    ///     .call(&[])?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn call(&self, params: &[Value]) -> CallResult<Vec<Value>> {
        if !self.signature.check_param_value_types(params) {
            Err(ResolveError::Signature {
                expected: (*self.signature).clone(),
                found: params.iter().map(|val| val.ty()).collect(),
            })?
        }

        let ctx = match self.func_index.local_or_import(&self.module.info) {
            LocalOrImport::Local(_) => self.instance_inner.ctx.get() as *mut vm::Ctx,
            LocalOrImport::Import(imported_func_index) => {
                self.instance_inner.import_backing.vm_functions[imported_func_index].ctx
            }
        };

        let token = Token::generate();

        let returns = self.module.protected_caller.call(
            &self.module,
            self.func_index,
            params,
            &self.instance_inner.import_backing,
            ctx,
            token,
        )?;

        Ok(returns)
    }

    pub fn signature(&self) -> &FuncSig {
        &*self.signature
    }

    pub fn raw(&self) -> *const vm::Func {
        match self.func_index.local_or_import(&self.module.info) {
            LocalOrImport::Local(local_func_index) => self
                .module
                .func_resolver
                .get(self.module, local_func_index)
                .unwrap()
                .as_ptr(),
            LocalOrImport::Import(import_func_index) => {
                self.instance_inner.import_backing.vm_functions[import_func_index].func
            }
        }
    }
}
