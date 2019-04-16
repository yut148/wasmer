use crate::module::ResourceIndex;

pub struct Allowed {
    pub float_ops: bool,
    pub indirect_calls: bool,
    _non_exhaustive: (),
}

impl Default for Allowed {
    fn default() -> Self {
        Self {
            float_ops: true,
            indirect_calls: true,
            _non_exhaustive: (),
        }
    }
}

pub struct Metering {
    _non_exhaustive: (),
}

impl Default for Metering {
    fn default() -> Self {
        Self {
            _non_exhaustive: (),
        }
    }
}

/// Configuration data for the compiler
pub struct CompileConfig<'a> {
    pub symbol_map: Option<Box<dyn Fn(ResourceIndex) -> Option<String>>>,
    pub metering: &'a Metering,
    pub allowed: &'a Allowed,
}
