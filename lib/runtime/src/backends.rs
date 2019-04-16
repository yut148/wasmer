use wasmer_runtime_core::{config::CompileConfig, error::CompileResult, module::Module};

pub trait Backend {
    fn compile(wasm: &[u8], config: CompileConfig) -> CompileResult<Module>;
}

pub struct SinglePass {
    _private: (),
}

impl Backend for SinglePass {
    fn compile(wasm: &[u8], config: CompileConfig) -> CompileResult<Module> {
        unimplemented!()
    }
}

pub struct Cranelift {
    _private: (),
}

impl Backend for Cranelift {
    fn compile(wasm: &[u8], config: CompileConfig) -> CompileResult<Module> {
        unimplemented!()
    }
}

pub struct Llvm {
    _private: (),
}

impl Backend for Llvm {
    fn compile(wasm: &[u8], config: CompileConfig) -> CompileResult<Module> {
        unimplemented!()
    }
}
