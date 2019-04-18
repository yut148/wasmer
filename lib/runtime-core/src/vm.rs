pub use crate::backing::{ImportBacking, LocalBacking};
use crate::{
    memory::Memory,
    module::{ModuleInner, ResourceIndex},
    structures::TypedIndex,
    types::{LocalOrImport, MemoryIndex},
};
use std::{ffi::c_void, mem, ptr};

use hashbrown::HashMap;

/// The context of the currently running WebAssembly instance.
///
///
#[derive(Debug)]
#[repr(C)]
pub struct Ctx<Data = ()> {
    // `internal` must be the first field of `Ctx`.
    pub(crate) internal: InternalCtx,

    pub(crate) local_functions: *const *const Func,

    local_backing: *mut LocalBacking,
    import_backing: *mut ImportBacking,
    pub(crate) module: *const ModuleInner,

    pub data: Data,
}

/// The internal context of the currently running WebAssembly instance.
///
///
#[doc(hidden)]
#[derive(Debug)]
#[repr(C)]
pub struct InternalCtx {
    /// A pointer to an array of locally-defined memories, indexed by `MemoryIndex`.
    pub memories: *mut *mut LocalMemory,

    /// A pointer to an array of locally-defined tables, indexed by `TableIndex`.
    pub tables: *mut *mut LocalTable,

    /// A pointer to an array of locally-defined globals, indexed by `GlobalIndex`.
    pub globals: *mut *mut LocalGlobal,

    /// A pointer to an array of imported memories, indexed by `MemoryIndex,
    pub imported_memories: *mut *mut LocalMemory,

    /// A pointer to an array of imported tables, indexed by `TableIndex`.
    pub imported_tables: *mut *mut LocalTable,

    /// A pointer to an array of imported globals, indexed by `GlobalIndex`.
    pub imported_globals: *mut *mut LocalGlobal,

    /// A pointer to an array of imported functions, indexed by `FuncIndex`.
    pub imported_funcs: *mut ImportedFunc,

    /// A pointer to an array of signature ids. Conceptually, this maps
    /// from a static, module-local signature id to a runtime-global
    /// signature id. This is used to allow call-indirect to other
    /// modules safely.
    pub dynamic_sigindices: *const SigId,
}

impl<Data> Ctx<Data> {
    pub(crate) unsafe fn new(
        local_backing: &mut LocalBacking,
        import_backing: &mut ImportBacking,
        module: &ModuleInner,
        data: Data,
    ) -> Self {
        Self {
            internal: InternalCtx {
                memories: local_backing.vm_memories.as_mut_ptr(),
                tables: local_backing.vm_tables.as_mut_ptr(),
                globals: local_backing.vm_globals.as_mut_ptr(),

                imported_memories: import_backing.vm_memories.as_mut_ptr(),
                imported_tables: import_backing.vm_tables.as_mut_ptr(),
                imported_globals: import_backing.vm_globals.as_mut_ptr(),
                imported_funcs: import_backing.vm_functions.as_mut_ptr(),

                dynamic_sigindices: local_backing.dynamic_sigindices.as_ptr(),
            },
            local_functions: local_backing.local_functions.as_ptr(),

            local_backing,
            import_backing,
            module,

            data,
        }
    }

    /// This exposes the specified memory of the WebAssembly instance
    /// as a immutable slice.
    ///
    /// WebAssembly will soon support multiple linear memories, so this
    /// forces the user to specify.
    ///
    /// # Usage:
    ///
    /// ```
    /// # use wasmer_runtime_core::{
    /// #     vm::Ctx,
    /// # };
    /// fn read_memory(ctx: &Ctx) -> u8 {
    ///     let first_memory = ctx.memory(0);
    ///     // Read the first byte of that linear memory.
    ///     first_memory.view()[0].get()
    /// }
    /// ```
    pub fn memory(&self, mem_index: u32) -> &Memory {
        let module = unsafe { &*self.module };
        let mem_index = MemoryIndex::new(mem_index as usize);
        match mem_index.local_or_import(&module.info) {
            LocalOrImport::Local(local_mem_index) => unsafe {
                let local_backing = &*self.local_backing;
                &local_backing.memories[local_mem_index]
            },
            LocalOrImport::Import(import_mem_index) => unsafe {
                let import_backing = &*self.import_backing;
                &import_backing.memories[import_mem_index]
            },
        }
    }
}

#[doc(hidden)]
impl Ctx<()> {
    #[allow(clippy::erasing_op)] // TODO
    pub fn offset_memories() -> u8 {
        0 * (mem::size_of::<usize>() as u8)
    }

    pub fn offset_tables() -> u8 {
        1 * (mem::size_of::<usize>() as u8)
    }

    pub fn offset_globals() -> u8 {
        2 * (mem::size_of::<usize>() as u8)
    }

    pub fn offset_imported_memories() -> u8 {
        3 * (mem::size_of::<usize>() as u8)
    }

    pub fn offset_imported_tables() -> u8 {
        4 * (mem::size_of::<usize>() as u8)
    }

    pub fn offset_imported_globals() -> u8 {
        5 * (mem::size_of::<usize>() as u8)
    }

    pub fn offset_imported_funcs() -> u8 {
        6 * (mem::size_of::<usize>() as u8)
    }

    pub fn offset_signatures() -> u8 {
        7 * (mem::size_of::<usize>() as u8)
    }

    pub fn offset_local_functions() -> u8 {
        8 * (mem::size_of::<usize>() as u8)
    }
}

enum InnerFunc {}
/// Used to provide type safety (ish) for passing around function pointers.
/// The typesystem ensures this cannot be dereferenced since an
/// empty enum cannot actually exist.
#[repr(C)]
pub struct Func(InnerFunc);

/// An imported function, which contains the ctx that owns this function.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct ImportedFunc {
    pub func: *const Func,
    pub ctx: *mut Ctx,
}

impl ImportedFunc {
    #[allow(clippy::erasing_op)] // TODO
    pub fn offset_func() -> u8 {
        0 * (mem::size_of::<usize>() as u8)
    }

    pub fn offset_ctx() -> u8 {
        1 * (mem::size_of::<usize>() as u8)
    }

    pub fn size() -> u8 {
        mem::size_of::<Self>() as u8
    }
}

/// Definition of a table used by the VM. (obviously)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct LocalTable {
    /// pointer to the elements in the table.
    pub base: *mut u8,
    /// Number of elements in the table (NOT necessarily the size of the table in bytes!).
    pub count: usize,
    /// The table that this represents. At the moment, this can only be `*mut AnyfuncTable`.
    pub table: *mut (),
}

impl LocalTable {
    #[allow(clippy::erasing_op)] // TODO
    pub fn offset_base() -> u8 {
        0 * (mem::size_of::<usize>() as u8)
    }

    pub fn offset_count() -> u8 {
        1 * (mem::size_of::<usize>() as u8)
    }

    pub fn size() -> u8 {
        mem::size_of::<Self>() as u8
    }
}

/// Definition of a memory used by the VM.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct LocalMemory {
    /// Pointer to the bottom of this linear memory.
    pub base: *mut u8,
    /// Current size of this linear memory in bytes.
    pub bound: usize,
    /// The actual memory that this represents.
    /// This is either `*mut DynamicMemory`, `*mut StaticMemory`,
    /// or `*mut SharedStaticMemory`.
    pub memory: *mut (),
}

impl LocalMemory {
    #[allow(clippy::erasing_op)] // TODO
    pub fn offset_base() -> u8 {
        0 * (mem::size_of::<usize>() as u8)
    }

    pub fn offset_bound() -> u8 {
        1 * (mem::size_of::<usize>() as u8)
    }

    pub fn size() -> u8 {
        mem::size_of::<Self>() as u8
    }
}

/// Definition of a global used by the VM.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct LocalGlobal {
    pub data: u64,
}

impl LocalGlobal {
    #[allow(clippy::erasing_op)] // TODO
    pub fn offset_data() -> u8 {
        0 * (mem::size_of::<usize>() as u8)
    }

    pub fn null() -> Self {
        Self { data: 0 }
    }

    pub fn size() -> u8 {
        mem::size_of::<Self>() as u8
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct SigId(pub u32);

/// Caller-checked anyfunc
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Anyfunc {
    pub func: *const Func,
    pub ctx: *mut Ctx,
    pub sig_id: SigId,
}

impl Anyfunc {
    pub fn null() -> Self {
        Self {
            func: ptr::null(),
            ctx: ptr::null_mut(),
            sig_id: SigId(u32::max_value()),
        }
    }

    #[allow(clippy::erasing_op)] // TODO
    pub fn offset_func() -> u8 {
        0 * (mem::size_of::<usize>() as u8)
    }

    pub fn offset_ctx() -> u8 {
        1 * (mem::size_of::<usize>() as u8)
    }

    pub fn offset_sig_id() -> u8 {
        2 * (mem::size_of::<usize>() as u8)
    }

    pub fn size() -> u8 {
        mem::size_of::<Self>() as u8
    }
}

#[cfg(test)]
mod vm_offset_tests {
    use super::{Anyfunc, Ctx, ImportedFunc, InternalCtx, LocalGlobal, LocalMemory, LocalTable};

    #[test]
    fn ctx() {
        assert_eq!(0usize, offset_of!(Ctx => internal).get_byte_offset(),);

        assert_eq!(
            Ctx::offset_memories() as usize,
            offset_of!(InternalCtx => memories).get_byte_offset(),
        );

        assert_eq!(
            Ctx::offset_tables() as usize,
            offset_of!(InternalCtx => tables).get_byte_offset(),
        );

        assert_eq!(
            Ctx::offset_globals() as usize,
            offset_of!(InternalCtx => globals).get_byte_offset(),
        );

        assert_eq!(
            Ctx::offset_imported_memories() as usize,
            offset_of!(InternalCtx => imported_memories).get_byte_offset(),
        );

        assert_eq!(
            Ctx::offset_imported_tables() as usize,
            offset_of!(InternalCtx => imported_tables).get_byte_offset(),
        );

        assert_eq!(
            Ctx::offset_imported_globals() as usize,
            offset_of!(InternalCtx => imported_globals).get_byte_offset(),
        );

        assert_eq!(
            Ctx::offset_imported_funcs() as usize,
            offset_of!(InternalCtx => imported_funcs).get_byte_offset(),
        );

        assert_eq!(
            Ctx::offset_local_functions() as usize,
            offset_of!(Ctx => local_functions).get_byte_offset(),
        );
    }

    #[test]
    fn imported_func() {
        assert_eq!(
            ImportedFunc::offset_func() as usize,
            offset_of!(ImportedFunc => func).get_byte_offset(),
        );

        assert_eq!(
            ImportedFunc::offset_ctx() as usize,
            offset_of!(ImportedFunc => ctx).get_byte_offset(),
        );
    }

    #[test]
    fn local_table() {
        assert_eq!(
            LocalTable::offset_base() as usize,
            offset_of!(LocalTable => base).get_byte_offset(),
        );

        assert_eq!(
            LocalTable::offset_count() as usize,
            offset_of!(LocalTable => count).get_byte_offset(),
        );
    }

    #[test]
    fn local_memory() {
        assert_eq!(
            LocalMemory::offset_base() as usize,
            offset_of!(LocalMemory => base).get_byte_offset(),
        );

        assert_eq!(
            LocalMemory::offset_bound() as usize,
            offset_of!(LocalMemory => bound).get_byte_offset(),
        );
    }

    #[test]
    fn local_global() {
        assert_eq!(
            LocalGlobal::offset_data() as usize,
            offset_of!(LocalGlobal => data).get_byte_offset(),
        );
    }

    #[test]
    fn cc_anyfunc() {
        assert_eq!(
            Anyfunc::offset_func() as usize,
            offset_of!(Anyfunc => func).get_byte_offset(),
        );

        assert_eq!(
            Anyfunc::offset_ctx() as usize,
            offset_of!(Anyfunc => ctx).get_byte_offset(),
        );

        assert_eq!(
            Anyfunc::offset_sig_id() as usize,
            offset_of!(Anyfunc => sig_id).get_byte_offset(),
        );
    }
}

#[cfg(test)]
mod vm_ctx_tests {}
