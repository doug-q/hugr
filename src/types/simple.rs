//! Dataflow types

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::{
    custom::{CustomType, CustomTypeTrait},
    Signature,
};
use crate::resource::ResourceSet;

/// A type that represents concrete data.
///
/// TODO: We define a flat enum for efficiency, but we could maybe split the
/// linear types into a nested enum instead.
///
/// TODO: Derive pyclass
///
/// TODO: Complete missing types
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum SimpleType {
    Variable(String), // TODO: How are variables represented?
    Int,
    Bool,
    F64,
    Quat64,
    Angle,
    Graph {
        resources: ResourceSet,
        signature: Signature,
    },
    Pair(Box<SimpleType>, Box<SimpleType>),
    List(Box<SimpleType>),

    // Linear types
    Qubit,
    Money,
    //
    Resource(ResourceSet),
    /// An opaque operation that can be downcasted by the extensions that define it.
    Opaque(CustomType),
}

/// Custom PartialEq implementation required to compare `DataType::Opaque` variants.
impl PartialEq for SimpleType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Variable(l0), Self::Variable(r0)) => l0 == r0,
            (
                Self::Graph {
                    resources: l_resources,
                    signature: l_signature,
                },
                Self::Graph {
                    resources: r_resources,
                    signature: r_signature,
                },
            ) => l_resources == r_resources && l_signature == r_signature,
            (Self::Pair(l0, l1), Self::Pair(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::List(l0), Self::List(r0)) => l0 == r0,
            (Self::Resource(l0), Self::Resource(r0)) => l0 == r0,
            (Self::Opaque(l0), Self::Opaque(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Eq for SimpleType {}

impl SimpleType {
    pub fn is_linear(&self) -> bool {
        match self {
            Self::Qubit | Self::Money => true,
            Self::Opaque(opaque) => opaque.is_linear(),
            _ => false,
        }
    }
}

impl Default for SimpleType {
    fn default() -> Self {
        Self::Qubit
    }
}

/// List of types, used for function signatures.
#[derive(Clone, PartialEq, Eq, Debug, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "pyo3", pyclass)]
#[non_exhaustive]
pub struct RowType {
    /// The datatypes in the row.
    pub types: Vec<SimpleType>,
}

#[cfg_attr(feature = "pyo3", pymethods)]
impl RowType {
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.types.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    #[inline(always)]
    pub fn purely_linear(&self) -> bool {
        self.types.iter().all(|typ| typ.is_linear())
    }

    #[inline(always)]
    pub fn purely_classical(&self) -> bool {
        !self
            .types
            .iter()
            .any(|typ| matches!(typ, SimpleType::Qubit | SimpleType::Money))
    }
}
impl RowType {
    /// Iterator over the types in the row.
    pub fn iter(&self) -> impl Iterator<Item = &SimpleType> {
        self.types.iter()
    }

    /// Mutable iterator over the types in the row.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut SimpleType> {
        self.types.iter_mut()
    }
}

impl RowType {
    pub fn new(types: impl Into<Vec<SimpleType>>) -> Self {
        Self {
            types: types.into(),
        }
    }
}

impl<T> From<T> for RowType
where
    T: Into<Vec<SimpleType>>,
{
    fn from(types: T) -> Self {
        Self::new(types.into())
    }
}

impl IntoIterator for RowType {
    type Item = SimpleType;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.types.into_iter()
    }
}
