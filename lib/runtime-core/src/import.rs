use crate::export::{Export, ExportIter};
use crate::instance::Instance;
use hashbrown::{
    hash_map::{Entry, Iter as HashMapIter},
    HashMap,
};
use std::collections::VecDeque;
use std::{
    cell::{Ref, RefCell},
    ffi::c_void,
    marker::PhantomData,
    rc::Rc,
    iter,
};

pub enum NamespaceItem<'a, Data = ()> {
    Instance(Instance<'a, Data>),
    Namespace(Namespace<'a, Data>),
}

impl<'a, Data> NamespaceItem<'a, Data> {
    pub fn get_export(&self, name: &str) -> Option<Export> {
        match self {
            NamespaceItem::Instance(instance) => instance.export(name),
            NamespaceItem::Namespace(namespace) => namespace.get_export(name),
        }
    }

    pub fn exports(&self) -> impl Iterator<Item = (&str, Export)> {
        enum Iter<'a, Data> {
            Inst(ExportIter<'a, Data>),
            Ns(HashMapIter<'a, String, Export<'a>>),
        }
        
        let mut export_iter = match self {
            NamespaceItem::Instance(inst) => Iter::Inst(inst.exports()),
            NamespaceItem::Namespace(ns) => Iter::Ns(ns.map.iter()),
        };

        iter::from_fn(move || {
            match &mut export_iter {
                Iter::Inst(iter) => iter.next(),
                Iter::Ns(ns) => ns.next().map(|(name, export)| (&**name, export.clone())),
            }
        })
    }

    pub fn try_insert(&mut self, name: impl Into<String>, export: Export<'a>) -> Result<(), ()> {
        match self {
            NamespaceItem::Instance(_) => Err(()),
            NamespaceItem::Namespace(ns) => {
                ns.insert(name, export);
                Ok(())
            }
        }
    }
}

impl<'a, Data> From<Instance<'a, Data>> for NamespaceItem<'a, Data> {
    fn from(instance: Instance<'a, Data>) -> Self {
        NamespaceItem::Instance(instance)
    }
}

impl<'a, Data> From<Namespace<'a, Data>> for NamespaceItem<'a, Data> {
    fn from(namespace: Namespace<'a, Data>) -> Self {
        NamespaceItem::Namespace(namespace)
    }
}

pub trait IsExport<'a, Data> {
    fn to_export(&self) -> Export<'a>;
}

impl<'a, Data> IsExport<'a, Data> for Export<'a> {
    fn to_export(&self) -> Export<'a> {
        self.clone()
    }
}

/// All of the import data used when instantiating.
///
/// It's suggested that you use the [`imports!`] macro
/// instead of creating an `ImportObject` by hand.
///
/// [`imports!`]: macro.imports.html
///
/// # Usage:
/// ```
/// # use wasmer_runtime_core::{imports, func};
/// # use wasmer_runtime_core::vm::Ctx;
/// let import_object = imports! {
///     "env" => {
///         "foo" => func!(foo),
///     },
/// };
///
/// fn foo(_: &mut Ctx, n: i32) -> i32 {
///     n
/// }
/// ```
pub struct ImportObject<'a, Data = ()> {
    map: HashMap<String, NamespaceItem<'a, Data>>,
    state_creator: Rc<dyn Fn() -> Data>,
}

impl<'a> ImportObject<'a, ()> {
    /// Create a new `ImportObject`.  
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            state_creator: Rc::new(|| ()),
        }
    }
}

impl<'a, Data> ImportObject<'a, Data> {
    pub fn new_with_data(state_creator: impl Fn() -> Data + 'static) -> Self {
        Self {
            map: HashMap::new(),
            state_creator: Rc::new(state_creator),
        }
    }

    pub(crate) fn create_state(&self) -> Data {
        self.state_creator.as_ref()()
    }

    /// Register anything that implements Into<NamespaceItem> as a namespace.
    ///
    /// # Usage:
    /// ```
    /// # use wasmer_runtime_core::Instance;
    /// # use wasmer_runtime_core::import::{ImportObject, Namespace};
    /// fn register(instance: Instance, namespace: Namespace) {
    ///     let mut import_object = ImportObject::new();
    ///
    ///     import_object.register("namespace0", instance);
    ///     import_object.register("namespace1", namespace);
    ///     // ...
    /// }
    /// ```
    pub fn register<S, N>(&mut self, name: S, namespace: N) -> Option<NamespaceItem<'a, Data>>
    where
        S: Into<String>,
        N: Into<NamespaceItem<'a, Data>>,
    {
        match self.map.entry(name.into()) {
            Entry::Vacant(empty) => {
                empty.insert(namespace.into());
                None
            }
            Entry::Occupied(mut occupied) => Some(occupied.insert(namespace.into())),
        }
    }

    pub fn get_namespace(&self, namespace: &str) -> Option<&NamespaceItem<'a, Data>> {
        self.map.get(namespace)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, impl Iterator<Item = (&str, Export)>)> {
        self.map.iter().map(|(name, ns)| (&**name, ns.exports()))
    }
}

impl<'a, Data: 'static, InnerIter: Iterator<Item = (&'a str, Export<'a>)>>
    Extend<(&'a str, InnerIter)> for ImportObject<'a, Data>
{
    fn extend<T: IntoIterator<Item = (&'a str, InnerIter)>>(&mut self, iter: T) {
        for (ns, inner_iter) in iter.into_iter() {
            for (field, export) in inner_iter {
                if let Some(like_ns) = self.map.get_mut(ns) {
                    let _ = like_ns.try_insert(field.to_string(), export);
                } else {
                    let mut new_ns = Namespace::new();
                    new_ns.insert(field.to_string(), export);
                    self.map.insert(ns.to_string(), new_ns.into());
                }
            }
        }
    }
}

pub struct Namespace<'a, Data = ()> {
    map: HashMap<String, Export<'a>>,
    _marker: PhantomData<Data>,
}

impl<'a, Data> Namespace<'a, Data> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            _marker: PhantomData,
        }
    }

    pub fn insert<S, E>(&mut self, name: S, export: E) -> Option<Export<'a>>
    where
        S: Into<String>,
        E: IsExport<'a, Data>,
    {
        self.map.insert(name.into(), export.to_export())
    }

    fn get_export(&'a self, name: &str) -> Option<Export<'a>> {
        self.map.get(name).cloned()
    }
}

#[cfg(test)]
mod test {
    use crate::export::Export;
    use crate::global::Global;
    use crate::types::Value;

    #[test]
    fn extending_works() {
        let imports2 = imports! {
            "dog" => {
                "small" => Global::new(Value::I32(2)),
            },
            "cat" => {
                "small" => Global::new(Value::I32(3)),
            },
        };

        let mut imports1 = imports! {
            "dog" => {
                "happy" => Global::new(Value::I32(0)),
            },
        };

        imports1.extend(imports2.iter());

        let cat_ns = imports1.get_namespace("cat").unwrap();
        assert!(cat_ns.get_export("small").is_some());

        let dog_ns = imports1.get_namespace("dog").unwrap();
        assert!(dog_ns.get_export("happy").is_some());
        assert!(dog_ns.get_export("small").is_some());
    }

    #[test]
    fn extending_conflict_overwrites() {
        let imports2 = imports! {
            "dog" => {
                "happy" => Global::new(Value::I32(4)),
            },
        };

        let mut imports1 = imports! {
            "dog" => {
                "happy" => Global::new(Value::I32(0)),
            },
        };

        imports1.extend(imports2.iter());
        let dog_ns = imports1.get_namespace("dog").unwrap();

        assert!(
            if let Export::Global(happy_dog_global) = dog_ns.get_export("happy").unwrap() {
                happy_dog_global.get() == Value::I32(4)
            } else {
                false
            }
        );
    }
}
