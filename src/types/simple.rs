//! Dataflow types

use std::borrow::Cow;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use smol_str::SmolStr;

use super::{custom::CustomType, Signature};
use crate::resource::ResourceSet;

/// A type that represents concrete data.
///
/// TODO: Derive pyclass
///
/// TODO: Compare performance vs flattening this into a single enum
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum SimpleType {
    Classic(ClassicType),
    Quantum(QuantumType),
}

/// A type that represents concrete classical data.
///
/// Uses `Box`es on most variants to reduce the memory footprint.
///
/// TODO: Derive pyclass
#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum ClassicType {
    Variable(SmolStr),
    Nat,
    Int,
    #[default]
    Bit,
    Graph(Box<(ResourceSet, Signature)>),
    Pair(Box<(ClassicType, ClassicType)>),
    List(Box<ClassicType>),
    Map(Box<(ClassicType, ClassicType)>),
    Struct(Box<TypeRow>),
    /// An opaque operation that can be downcasted by the extensions that define it.
    Opaque(CustomType),
}

/// A type that represents concrete quantum data.
///
/// TODO: Derive pyclass
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum QuantumType {
    #[default]
    Qubit,
    Money,
    Array(Box<QuantumType>, usize),
}

impl SimpleType {
    pub fn is_linear(&self) -> bool {
        matches!(self, Self::Quantum(_))
    }

    pub fn is_classical(&self) -> bool {
        matches!(self, Self::Classic(_))
    }
}

impl Default for SimpleType {
    fn default() -> Self {
        Self::Quantum(Default::default())
    }
}

impl From<ClassicType> for SimpleType {
    fn from(typ: ClassicType) -> Self {
        Self::Classic(typ)
    }
}

impl From<QuantumType> for SimpleType {
    fn from(typ: QuantumType) -> Self {
        Self::Quantum(typ)
    }
}

/// List of types, used for function signatures.
#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "pyo3", pyclass)]
#[non_exhaustive]
pub struct TypeRow {
    /// The datatypes in the row.
    types: Option<Cow<'static, [SimpleType]>>,
}

#[cfg_attr(feature = "pyo3", pymethods)]
impl TypeRow {
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.types.as_ref().map_or(0, |types| types.len())
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.types.is_none()
    }

    #[inline(always)]
    pub fn purely_linear(&self) -> bool {
        let Some(types) = &self.types else {return false};
        types.iter().all(|typ| typ.is_linear())
    }

    #[inline(always)]
    pub fn purely_classical(&self) -> bool {
        let Some(types) = &self.types else {return false};
        !types.iter().all(SimpleType::is_classical)
    }
}
impl TypeRow {
    /// Create a new empty row.
    pub const fn new() -> Self {
        Self { types: None }
    }

    /// Create a new row from a Cow slice of types.
    ///
    /// See [`type_row!`] for a more ergonomic way to create a statically allocated rows.
    ///
    /// [`type_row!`]: crate::macros::type_row
    pub fn from(types: impl Into<Cow<'static, [SimpleType]>>) -> Self {
        let types = types.into();
        Self {
            types: if types.is_empty() { None } else { Some(types) },
        }
    }

    /// Iterator over the types in the row.
    pub fn iter(&self) -> impl Iterator<Item = &SimpleType> {
        self.types.iter().flat_map(|types| types.iter())
    }

    /// Mutable iterator over the types in the row.
    ///
    /// This will allocate if the row is empty.
    ///
    /// TODO: If the vector is emptied, it should set `types` to `None`. (Or we should modify all the methods to handle empty vectors.)
    pub fn to_mut(&mut self) -> &mut Vec<SimpleType> {
        self.types
            .get_or_insert_with(|| Cow::Owned(Vec::new()))
            .to_mut()
    }
}

impl Default for TypeRow {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<T> for TypeRow
where
    T: Into<Cow<'static, [SimpleType]>>,
{
    fn from(types: T) -> Self {
        Self::from(types.into())
    }
}
