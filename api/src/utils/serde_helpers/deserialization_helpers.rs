use serde::{de::Deserializer, Deserialize};
use serde_json::Value;

pub fn deserialize_double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    
    match value {
        Value::Null => Ok(Some(None)),      // explicit null
        Value::Object(obj) if obj.is_empty() => Ok(None), // empty object
        _ => {
            match T::deserialize(value) {
                Ok(val) => Ok(Some(Some(val))),
                Err(_) => Ok(None)
            }
        }
    }
}
