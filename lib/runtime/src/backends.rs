use wasmer_runtime_core::{config::CompileConfig, error::CompileResult, module::Module, backend::Compiler};

pub trait Backend {
    fn compile(wasm: &[u8], config: CompileConfig) -> CompileResult<Module>;
}

#[cfg(feature = "backend:singlepass")]
pub struct SinglePass {
    _private: (),
}

#[cfg(feature = "backend:singlepass")]
impl Backend for SinglePass {
    fn compile(wasm: &[u8], config: CompileConfig) -> CompileResult<Module> {
        use wasmer_singlepass_backend::SinglePassCompiler;
        SinglePassCompiler::new().compile(wasm, config)
    }
}

#[cfg(feature = "backend:cranelift")]
pub struct Cranelift {
    _private: (),
}

#[cfg(feature = "backend:cranelift")]
impl Backend for Cranelift {
    fn compile(wasm: &[u8], config: CompileConfig) -> CompileResult<Module> {
        use wasmer_clif_backend::CraneliftCompiler;
        CraneliftCompiler::new().compile(wasm, config)
    }
}

#[cfg(feature = "backend:llvm")]
pub struct Llvm {
    _private: (),
}

#[cfg(feature = "backend:llvm")]
impl Backend for Llvm {
    fn compile(wasm: &[u8], config: CompileConfig) -> CompileResult<Module> {
        use wasmer_llvm_backend::LLVMCompiler;
        LLVMCompiler::new().compile(wasm, config)
    }
}
