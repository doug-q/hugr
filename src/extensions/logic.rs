//! Basic logical operations.

use std::collections::HashMap;

use itertools::Itertools;
use smol_str::SmolStr;

use crate::{
    ops,
    resource::ResourceSet,
    types::{
        type_param::{TypeArg, TypeArgError, TypeParam},
        HashableType, SimpleType,
    },
    Resource,
};

/// Name of resource false value.
pub const FALSE_NAME: &str = "FALSE";
/// Name of resource true value.
pub const TRUE_NAME: &str = "TRUE";

/// The resource identifier.
pub const fn resource_id() -> SmolStr {
    SmolStr::new_inline("Logic")
}

/// Construct a boolean type.
pub fn bool_type() -> SimpleType {
    SimpleType::new_simple_predicate(2)
}

/// Resource for basic logical operations.
pub fn resource() -> Resource {
    const H_INT: TypeParam = TypeParam::Value(HashableType::Int(8));
    let mut resource = Resource::new(resource_id());

    resource
        .add_op_custom_sig(
            "Not",
            "logical 'not'".into(),
            vec![],
            HashMap::default(),
            |_arg_values: &[TypeArg]| {
                Ok((
                    vec![bool_type()].into(),
                    vec![bool_type()].into(),
                    ResourceSet::default(),
                ))
            },
        )
        .unwrap();

    resource
        .add_op_custom_sig(
            "And",
            "logical 'and'".into(),
            vec![H_INT],
            HashMap::default(),
            |arg_values: &[TypeArg]| {
                let a = arg_values.iter().exactly_one().unwrap();
                let n: u128 = match a {
                    TypeArg::Int(n) => *n,
                    _ => {
                        return Err(TypeArgError::TypeMismatch(a.clone(), H_INT).into());
                    }
                };
                Ok((
                    vec![bool_type(); n as usize].into(),
                    vec![bool_type()].into(),
                    ResourceSet::default(),
                ))
            },
        )
        .unwrap();

    resource
        .add_op_custom_sig(
            "Or",
            "logical 'or'".into(),
            vec![H_INT],
            HashMap::default(),
            |arg_values: &[TypeArg]| {
                let a = arg_values.iter().exactly_one().unwrap();
                let n: u128 = match a {
                    TypeArg::Int(n) => *n,
                    _ => {
                        return Err(TypeArgError::TypeMismatch(a.clone(), H_INT).into());
                    }
                };
                Ok((
                    vec![bool_type(); n as usize].into(),
                    vec![bool_type()].into(),
                    ResourceSet::default(),
                ))
            },
        )
        .unwrap();

    resource
        .add_value(FALSE_NAME, ops::Const::simple_predicate(0, 2))
        .unwrap();
    resource
        .add_value(TRUE_NAME, ops::Const::simple_predicate(1, 2))
        .unwrap();
    resource
}

#[cfg(test)]
mod test {
    use crate::{types::SimpleType, Resource};

    use super::{bool_type, resource, FALSE_NAME, TRUE_NAME};

    #[test]
    fn test_logic_resource() {
        let r: Resource = resource();
        assert_eq!(r.name(), "Logic");
        assert_eq!(r.num_operations(), 3);
    }

    #[test]
    fn test_values() {
        let r: Resource = resource();
        let false_val = r.get_value(FALSE_NAME).unwrap();
        let true_val = r.get_value(TRUE_NAME).unwrap();

        for v in [false_val, true_val] {
            let simpl: SimpleType = v.typed_value().const_type().clone().into();
            assert_eq!(simpl, bool_type());
        }
    }
}
