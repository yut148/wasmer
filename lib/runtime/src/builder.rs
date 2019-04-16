pub use wasmer_runtime_core::config::{Allowed, Metering};
use wasmer_runtime_core::{
    config::CompileConfig,
    error::CompileResult,
    module::{Module, ResourceIndex},
};

use crate::backends::Backend;
use std::marker::PhantomData;

pub struct CompilerBuilder<B: Backend> {
    metering: Metering,
    allowed: Allowed,
    _phantom: PhantomData<B>,
}

impl<B: Backend> CompilerBuilder<B> {
    pub fn metering(&mut self, metering: Metering) -> &mut Self {
        self.metering = metering;
        self
    }

    pub fn allowed(&mut self, allowed: Allowed) -> &mut Self {
        self.allowed = allowed;
        self
    }

    pub fn build(self) -> Compiler<B> {
        Compiler {
            metering: self.metering,
            allowed: self.allowed,
            _phantom: PhantomData,
        }
    }
}

pub struct ModuleBuilder<'a, B: Backend> {
    wasm: &'a [u8],
    metering: &'a Metering,
    allowed: &'a Allowed,
    symbol_mapper: Option<Box<dyn Fn(ResourceIndex) -> Option<String>>>,
    _phantom: PhantomData<B>,
}

impl<'a, B: Backend> ModuleBuilder<'a, B> {
    pub fn map_symbols(
        &mut self,
        mapper: impl Fn(ResourceIndex) -> Option<String> + 'static,
    ) -> &mut Self {
        self.symbol_mapper = Some(Box::new(mapper));
        self
    }

    pub fn compile(self) -> CompileResult<Module> {
        B::compile(
            self.wasm,
            CompileConfig {
                metering: self.metering,
                allowed: self.allowed,
                symbol_map: self.symbol_mapper,
            },
        )
    }
}

///
/// # Usage:
/// ```
/// # use wasmer_runtime::{Compiler, backends::SinglePass, Metering};
///
/// let compiler: Compiler<SinglePass> = Compiler::new()
///     .metering(Metering {
///         .. Metering::default(),
///     })
///     .build();
///
/// let module = compiler.module(&[])
///     .map_symbols(|resource_index| resource_map.get(resource_index))
///     .compile().unwrap();
///
///
/// ```
pub struct Compiler<B: Backend> {
    metering: Metering,
    allowed: Allowed,
    _phantom: PhantomData<B>,
}

impl<B: Backend> Compiler<B> {
    pub fn new() -> CompilerBuilder<B> {
        CompilerBuilder {
            metering: Default::default(),
            allowed: Default::default(),
            _phantom: PhantomData,
        }
    }

    pub fn module<'a>(&'a self, wasm: &'a [u8]) -> ModuleBuilder<'a, B> {
        ModuleBuilder {
            wasm,
            metering: &self.metering,
            allowed: &self.allowed,
            symbol_mapper: None,
            _phantom: PhantomData,
        }
    }
}
