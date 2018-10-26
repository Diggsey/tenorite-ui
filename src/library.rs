use std::collections::BTreeMap;
use std::sync::Arc;
use std::borrow::Cow;
use std::fmt;
use std::error::Error;

use serde_derive::{Serialize, Deserialize};

use crate::component::{AnyComponent, ComponentInfo};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComponentMetadata {
    pub id: Cow<'static, str>,
    pub name: Cow<'static, str>,
    pub category: Cow<'static, str>,
    pub description: Cow<'static, str>,
}

impl ComponentMetadata {
    pub fn new<
        S1: Into<Cow<'static, str>>,
        S2: Into<Cow<'static, str>>,
        S3: Into<Cow<'static, str>>,
        S4: Into<Cow<'static, str>>,
    >(id: S1, name: S2, category: S3, description: S4) -> Self {
        ComponentMetadata {
            id: id.into(),
            name: name.into(),
            category: category.into(),
            description: description.into(),
        }
    }
}

#[derive(Clone)]
struct ComponentEntry {
    metadata: Arc<ComponentMetadata>,
    factory: Arc<Fn() -> Box<AnyComponent> + Send + Sync + 'static>,
}

impl fmt::Debug for ComponentEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.metadata.fmt(f)
    }
}

#[derive(Clone, Default, Debug)]
pub struct Library {
    components: BTreeMap<String, ComponentEntry>
}

#[derive(Debug, Clone)]
pub struct MissingComponentError {
    pub id: String,
}

impl fmt::Display for MissingComponentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing component with id `{}`", self.id)
    }
}

impl Error for MissingComponentError { }

impl Library {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn list(&self) -> Vec<Arc<ComponentMetadata>> {
        self.components.values().map(|c| c.metadata.clone()).collect()
    }
    pub fn create(&self, id: &str) -> Result<ComponentInfo, MissingComponentError> {
        let entry = self.components.get(id)
            .ok_or_else(|| MissingComponentError { id: id.into() })?;
        Ok(ComponentInfo::new((entry.factory)(), entry.metadata.clone()))
    }
    pub fn extend(&mut self, other: Library) {
        self.components.extend(other.components.into_iter());
    }
    pub fn add<F: Fn() -> Box<AnyComponent> + Send + Sync + 'static>(&mut self, metadata: ComponentMetadata, f: F) {
        let id = metadata.id.clone().into_owned();
        self.components.insert(id, ComponentEntry {
            metadata: Arc::new(metadata),
            factory: Arc::new(f),
        });
    }
}
