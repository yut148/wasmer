use crate::{
    backing::ImportBacking,
    config::CompileConfig,
    error::CompileResult,
    error::RuntimeResult,
    module::{ModuleInner, ResourceIndex},
    typed_func::Wasm,
    types::{LocalFuncIndex, SigIndex},
    vm,
};

use crate::{
    cache::{Artifact, Error as CacheError},
    module::ModuleInfo,
    sys::Memory,
};
use std::{any::Any, ptr::NonNull};

use hashbrown::HashMap;

pub mod sys {
    pub use crate::sys::*;
}
pub use crate::sig_registry::SigRegistry;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Backend {
    Cranelift,
    Singlepass,
    LLVM,
}

pub trait Compiler {
    /// Compiles a `Module` from WebAssembly binary format.
    fn compile(&self, wasm: &[u8], config: CompileConfig) -> CompileResult<ModuleInner>;

    unsafe fn from_cache(&self, cache: Artifact) -> Result<ModuleInner, CacheError>;
}

pub trait RunnableModule: Send + Sync {
    /// This returns a pointer to the function designated by the `local_func_index`
    /// parameter.
    fn get_func(
        &self,
        info: &ModuleInfo,
        local_func_index: LocalFuncIndex,
    ) -> Option<NonNull<vm::Func>>;

    /// A wasm trampoline contains the necesarry data to dynamically call an exported wasm function.
    /// Given a particular signature index, we are returned a trampoline that is matched with that
    /// signature and an invoke function that can call the trampoline.
    fn get_trampoline(&self, info: &ModuleInfo, sig_index: SigIndex) -> Option<Wasm>;

    unsafe fn do_early_trap(&self, data: Box<dyn Any>) -> !;
}

pub trait CacheGen: Send + Sync {
    fn generate_cache(
        &self,
        module: &ModuleInner,
    ) -> Result<(Box<ModuleInfo>, Box<[u8]>, Memory), CacheError>;
}
