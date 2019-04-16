use crate::{
    global::Global, instance::InstanceInner, memory::Memory, module::ModuleInner,
    module::ResourceIndex, table::Table, types::FuncSig, vm,
};
use hashbrown::hash_map;
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Debug, Copy, Clone)]
pub enum Context {
    External(*mut vm::Ctx),
    Internal,
}

#[derive(Debug, Clone)]
pub enum Export<'a> {
    Function {
        func: FuncPointer,
        ctx: Context,
        signature: Arc<FuncSig>,
        _marker: PhantomData<&'a ()>,
    },
    Memory(Memory),
    Table(Table),
    Global(Global),
}

#[derive(Debug, Clone)]
pub struct FuncPointer(*const vm::Func);

impl FuncPointer {
    /// This needs to be unsafe because there is
    /// no way to check whether the passed function
    /// is valid and has the right signature.
    pub unsafe fn new(f: *const vm::Func) -> Self {
        FuncPointer(f)
    }

    pub(crate) fn inner(&self) -> *const vm::Func {
        self.0
    }
}

pub struct ExportIter<'a, Data = ()> {
    inner: &'a InstanceInner<Data>,
    iter: hash_map::Iter<'a, String, ResourceIndex>,
    module: &'a ModuleInner,
}

impl<'a, Data> ExportIter<'a, Data> {
    pub(crate) fn new(module: &'a ModuleInner, inner: &'a InstanceInner<Data>) -> Self {
        Self {
            inner,
            iter: module.info.exports.iter(),
            module,
        }
    }
}

impl<'a, Data> Iterator for ExportIter<'a, Data> {
    type Item = (&'a str, Export<'a>);
    fn next(&mut self) -> Option<(&'a str, Export<'a>)> {
        let (name, export_index) = self.iter.next()?;
        Some((
            name,
            self.inner.get_export_from_index(&self.module, export_index),
        ))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let num_exports = self.module.info.exports.len();
        (num_exports, Some(num_exports))
    }
}
