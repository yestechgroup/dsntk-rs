//! # Primitive type resolution

use crate::registry::{TypeEntry, TypeSource};
use dsntk_feel::FeelType;
use std::collections::HashMap;

/// Well-known primitive type names that resolve without a schema file.
const PRIMITIVES: &[(&str, FeelType)] = &[
  ("number", FeelType::Number),
  ("string", FeelType::String),
  ("boolean", FeelType::Boolean),
  ("date", FeelType::Date),
  ("date and time", FeelType::DateTime),
  ("time", FeelType::Time),
  ("days and time duration", FeelType::DaysAndTimeDuration),
  ("years and months duration", FeelType::YearsAndMonthsDuration),
];

/// Returns `true` if the given type name is a primitive FEEL type.
pub fn is_primitive(type_name: &str) -> bool {
  PRIMITIVES.iter().any(|(name, _)| *name == type_name)
}

/// Resolves a primitive type name to a `TypeEntry`.
/// Returns `None` if the name is not a known primitive.
pub fn resolve_primitive(type_name: &str) -> Option<TypeEntry> {
  PRIMITIVES.iter().find(|(name, _)| *name == type_name).map(|(name, feel_type)| TypeEntry {
    name: name.to_string(),
    feel_type: feel_type.clone(),
    source: TypeSource::Primitive,
    allowed_values: None,
    optional_fields: HashMap::new(),
  })
}
