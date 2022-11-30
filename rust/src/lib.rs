use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod error;
pub mod validator;
use error::*;
use pyo3::{exceptions::PyTypeError, prelude::*};
use validator::AS3Validator;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum AS3Data {
    Object(HashMap<String, Box<AS3Data>>),
    String(String),
    Boolean(bool),
    Integer(i64),
    Decimal(f64),
    List(Vec<AS3Data>),
    Null,
}

impl From<&serde_json::Value> for AS3Data {
    fn from(json: &serde_json::Value) -> AS3Data {
        match json {
            serde_json::Value::Object(inner) => AS3Data::Object(
                inner
                    .iter()
                    .map(|(key, value)| (key.clone(), Box::new(value.into())))
                    .collect(),
            ),
            serde_json::Value::Array(inner) => {
                AS3Data::List(inner.clone().iter().map(|e| e.into()).collect())
            }
            serde_json::Value::String(inner) => AS3Data::String(inner.clone()),
            serde_json::Value::Number(inner) => {
                if let Some(number) = inner.as_i64() {
                    AS3Data::Integer(number)
                } else {
                    AS3Data::Decimal(inner.as_f64().unwrap())
                }
            }
            serde_json::Value::Bool(inner) => AS3Data::Boolean(*inner),
            serde_json::Value::Null => AS3Data::Null,
        }
    }
}

#[pyfunction]
pub fn verify(data: String, validator_config: String) -> PyResult<()> {
    let data = AS3Data::from(&serde_json::from_str(&data).unwrap());
    let ym = serde_yaml::from_str(&validator_config).unwrap();
    let validator = AS3Validator::from(&ym).unwrap();
    match validator.validate(&data) {
        Ok(_) => Ok(()),
        Err(e) => Err(PyTypeError::new_err(e.to_string())),
    }
}

#[pymodule]
fn as3(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(verify, m)?)?;
    Ok(())
}

#[cfg(test)]
#[path = "integration_test.rs"]
mod test;
