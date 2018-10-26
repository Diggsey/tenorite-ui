use std::collections::BTreeMap;
use std::fmt;
use std::any::Any;
use std::error::Error;
use std::mem;
use std::borrow::Cow;
use std::sync::Arc;

use serde_derive::{Serialize, Deserialize};
use serde_json;
use serde::Serialize;

use crate::library::ComponentMetadata;

#[derive(Serialize, Debug, Clone)]
pub enum FieldType {
    Text {
        min_len: u32,
        max_len: u32,
    },
    Integer {
        min: u32,
        max: u32,
    },
    Enum {
        options: Vec<String>
    }
}

impl FieldType {
    pub fn for_enum<T: Serialize>(variants: &[T]) -> Self {
        let options = variants
            .into_iter()
            .map(|v| match serde_json::to_value(v) {
                Ok(serde_json::Value::String(s)) => s,
                _ => panic!("Variant did not serialize to a string")
            })
            .collect();
        FieldType::Enum { options }
    }
}

pub trait ReflectType {
    fn field_type() -> FieldType;
}

#[derive(Serialize, Debug, Clone)]
pub struct FieldSchema {
    pub read_only: bool,
    pub type_: FieldType,
    pub name: Cow<'static, str>,
    pub description: Option<Cow<'static, str>>,
}

pub type Schema = BTreeMap<Cow<'static, str>, FieldSchema>;

#[derive(Serialize, Debug, Clone)]
pub enum PropertyErrorReason {
    UnknownProperty,
    ReadOnlyProperty,
    InvalidValue {
        explanation: String
    },
}

#[derive(Serialize, Debug, Clone)]
pub struct PropertyError {
    pub name: String,
    pub reason: PropertyErrorReason,
}

impl fmt::Display for PropertyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.reason {
            PropertyErrorReason::UnknownProperty =>
                write!(f, "Unknown property `{}`", self.name),
            PropertyErrorReason::ReadOnlyProperty =>
                write!(f, "Property `{}` is read-only", self.name),
            PropertyErrorReason::InvalidValue { ref explanation } =>
                write!(f, "Invalid value for property `{}`: {}", self.name, explanation),            
        }
    }
}

impl Error for PropertyError {}

impl PropertyError {
    pub fn from_serde(error: serde_json::Error, name: &str) -> PropertyError {
        PropertyError {
            name: name.into(),
            reason: PropertyErrorReason::InvalidValue {
                explanation: error.to_string()
            }
        }
    }
    pub fn unknown(name: &str) -> PropertyError {
        PropertyError {
            name: name.into(),
            reason: PropertyErrorReason::UnknownProperty,
        }
    }
    pub fn read_only(name: &str) -> PropertyError {
        PropertyError {
            name: name.into(),
            reason: PropertyErrorReason::ReadOnlyProperty,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Pin {
    pub x: i32,
    pub y: i32,
    pub name: String,
    pub bits: u32,
}

#[derive(Serialize, Debug, Clone)]
pub struct Shape {
    pub width: i32,
    pub height: i32,
    pub pins: Vec<Pin>,
    pub image_name: Cow<'static, str>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Orientation {
    North, East, South, West
}

impl Orientation {
    fn map_point(&self, x: i32, y: i32, w: i32, h: i32) -> (i32, i32) {
        match self {
            Orientation::North => (x, y),
            Orientation::East => (w - y, x),
            Orientation::South => (w - x, h - y),
            Orientation::West => (y, h - x),
        }
    }
    fn map_shape(&self, mut shape: Shape) -> Shape {
        match self {
            Orientation::North | Orientation::South => {},
            Orientation::East | Orientation::West => mem::swap(&mut shape.width, &mut shape.height),
        };

        for pin in &mut shape.pins {
            let (x, y) = self.map_point(pin.x, pin.y, shape.width, shape.height);
            pin.x = x;
            pin.y = y;
        }

        shape
    }
}

impl ReflectType for Orientation {
    fn field_type() -> FieldType {
        use self::Orientation::*;
        FieldType::for_enum(&[North, East, South, West])
    }
}

pub trait Component: Any + fmt::Debug + CloneComponent {
    fn schema(&self) -> Schema;
    fn set_property(&mut self, name: &str, value: serde_json::Value) -> Result<(), PropertyError>;
    fn get_property(&self, name: &str) -> Option<serde_json::Value>;
    fn get_shape(&self) -> Shape;
}

pub trait AnyComponent: Component {
    fn as_any_mut(&mut self) -> &mut Any;
    fn as_any_ref(&self) -> &Any;
}

impl<T: Component> AnyComponent for T {
    fn as_any_mut(&mut self) -> &mut Any { self }
    fn as_any_ref(&self) -> &Any { self }
}

pub trait CloneComponent {
    fn clone_component(&self) -> Box<AnyComponent>;
}

impl<T: Clone + Component> CloneComponent for T {
    fn clone_component(&self) -> Box<AnyComponent> {
        Box::new(self.clone())
    }
}

pub struct ComponentInfo {
    component: Box<AnyComponent>,
    orientation: Orientation,
    x: i32,
    y: i32,
    metadata: Arc<ComponentMetadata>,
}

impl ComponentInfo {
    const ORIENTATION: &'static str = "orientation";
    pub(crate) fn new(component: Box<AnyComponent>, metadata: Arc<ComponentMetadata>) -> Self {
        Self {
            component,
            orientation: Orientation::North,
            x: 0,
            y: 0,
            metadata
        }
    }
    pub fn schema(&self) -> Schema {
        let mut s = self.component.schema();
        s.insert(Self::ORIENTATION.into(), FieldSchema {
            type_: Orientation::field_type(),
            read_only: false,
            name: "Orientation".into(),
            description: None,
        });
        s
    }
    pub fn set_property(&mut self, name: &str, value: serde_json::Value) -> Result<(), PropertyError> {
        match name {
            Self::ORIENTATION => {
                self.orientation = serde_json::from_value(value)
                    .map_err(|e| PropertyError::from_serde(e, name))?;
            },
            _ => self.component.set_property(name, value)?,
        }
        Ok(())
    }
    pub fn get_property(&self, name: &str) -> Option<serde_json::Value> {
        match name {
            Self::ORIENTATION => serde_json::to_value(self.orientation).ok(),
            _ => self.component.get_property(name),
        }
    }
    pub fn get_shape(&self) -> Shape {
        self.orientation.map_shape(self.component.get_shape())
    }
}
