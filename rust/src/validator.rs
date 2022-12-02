use crate::{
    error::{AS3ValidationError, As3JsonPath},
    AS3Data,
};

use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum AS3Validator {
    #[serde(rename(serialize = "Object"))]
    Object(HashMap<String, AS3Validator>),
    #[serde(rename(serialize = "String"))]
    String {
        regex: Option<String>,
        max_length: Option<i64>,
        min_length: Option<i64>,
    },
    #[serde(rename(serialize = "Integer"))]
    Integer {
        minimum: Option<i64>,
        maximum: Option<i64>,
    },
    #[serde(rename(serialize = "Decimal"))]
    Decimal {
        minimum: Option<f64>,
        maximum: Option<f64>,
    },
    #[serde(rename(serialize = "List"))]
    List(Box<AS3Validator>),
    #[serde(rename(serialize = "Map"))]
    Map {
        key_type: Box<AS3Validator>,
        value_type: Box<AS3Validator>,
    },
    #[serde(rename(serialize = "Bool"))]
    Boolean,
    #[serde(rename(serialize = "Date"))]
    Date,
    #[serde(rename(serialize = "Nullable"))]
    Nullable(Box<AS3Validator>),
}

impl AS3Validator {
    pub fn validate(&self, data: &AS3Data) -> Result<(), As3JsonPath<AS3ValidationError>> {
        self.check(data, &mut "ROOT".to_string())
    }

    fn check(
        &self,
        data: &AS3Data,
        path: &mut String,
    ) -> Result<(), As3JsonPath<AS3ValidationError>> {
        match (self, data) {
            (AS3Validator::Nullable(..), AS3Data::Null) => return Ok(()),
            (_, AS3Data::Null) => {
                return Err(As3JsonPath(
                    path.to_string(),
                    AS3ValidationError::NotNullableNull,
                ))
            }
            _ => {}
        };

        match (self, data) {
            (AS3Validator::Object(validator_inner), AS3Data::Object(data_inner)) => {
                let res: Vec<Result<(), As3JsonPath<AS3ValidationError>>> = validator_inner
                    .into_par_iter()
                    .map(|(validator_key, validator_value)| {
                        let mut temp_path = path.clone();
                        temp_path.push_str(" -> ");
                        temp_path.push_str(&validator_key.as_str());
                        if let Some(value_from_key) = data_inner.get(validator_key) {
                            return validator_value.check(value_from_key, &mut temp_path);
                        }
                        Err(As3JsonPath(
                            path.to_string(),
                            AS3ValidationError::MissingKey {
                                key: validator_key.clone(),
                            },
                        ))
                    })
                    .collect();

                match res
                    .into_iter()
                    .collect::<Result<Vec<()>, As3JsonPath<AS3ValidationError>>>()
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            }
            (
                AS3Validator::Map {
                    key_type,
                    value_type,
                },
                AS3Data::Object(data_inner),
            ) => {
                for (key_data, value_data) in data_inner {
                    let mut temp_path = path.clone();
                    temp_path.push_str(" -> ");
                    temp_path.push_str(&key_data.as_str());
                    match (
                        value_type.check(value_data, &mut temp_path),
                        AS3Validator::check_map_key_value(key_data, key_type, &mut temp_path),
                    ) {
                        (Ok(_), Ok(_)) => {}
                        (Err(e), _) => return Err(e),
                        (_, Err(e)) => {
                            return Err(As3JsonPath(
                                temp_path.to_string(),
                                AS3ValidationError::Generic(e),
                            ))
                        }
                    };
                }
                Ok(())
            }
            (AS3Validator::Integer { minimum, maximum }, AS3Data::Integer(number)) => {
                if let Some(minimum) = minimum {
                    if number < minimum {
                        return Err(As3JsonPath(
                            path.to_string(),
                            AS3ValidationError::MinimumInteger {
                                number: *number,
                                minimum: *minimum,
                            },
                        ));
                    }
                }

                if let Some(maximum) = maximum {
                    if number > maximum {
                        return Err(As3JsonPath(
                            path.to_string(),
                            AS3ValidationError::MaximumInteger {
                                number: *number,
                                maximum: *maximum,
                            },
                        ));
                    }
                }
                Ok(())
            }
            (AS3Validator::Decimal { minimum, maximum }, AS3Data::Decimal(number)) => {
                if let Some(minimum) = minimum {
                    if number < minimum {
                        return Err(As3JsonPath(
                            path.to_string(),
                            AS3ValidationError::MinimumDouble {
                                number: *number as f64,
                                minimum: *minimum as f64,
                            },
                        ));
                    }
                }

                if let Some(maximum) = maximum {
                    if number > maximum {
                        return Err(As3JsonPath(
                            path.to_string(),
                            AS3ValidationError::MinimumDouble {
                                number: *number as f64,
                                minimum: *maximum as f64,
                            },
                        ));
                    }
                }
                Ok(())
            }
            (
                AS3Validator::String {
                    regex,
                    max_length,
                    min_length,
                },
                AS3Data::String(string),
            ) => {
                if let Some(regex) = regex {
                    let re = Regex::new(regex).unwrap();
                    if !re.is_match(string) {
                        return Err(As3JsonPath(
                            path.to_string(),
                            AS3ValidationError::RegexError {
                                word: string.to_owned(),
                                regex: regex.to_owned(),
                            },
                        ));
                    }
                };
                if let Some(min_length) = min_length {
                    if string.len() < *min_length as usize {
                        return Err(As3JsonPath(
                            path.to_string(),
                            AS3ValidationError::MinimumString {
                                string: string.clone(),
                                current_lenght: string.len() as i64,
                                min_length: *min_length,
                            },
                        ));
                    }
                }

                if let Some(max_length) = max_length {
                    if string.len() > *max_length as usize {
                        return Err(As3JsonPath(
                            path.to_string(),
                            AS3ValidationError::MaximumString {
                                string: string.clone(),
                                current_lenght: string.len() as i64,
                                max_length: *max_length,
                            },
                        ));
                    }
                }

                Ok(())
            }
            (AS3Validator::List(items_type), AS3Data::List(items)) => {
                // Ok(items.iter().all(|item| items_type.check(item)))

                let res = items
                    .iter()
                    .map(|item| items_type.check(item, path))
                    .collect::<Vec<Result<(), As3JsonPath<AS3ValidationError>>>>();

                match res
                    .into_iter()
                    .collect::<Result<Vec<()>, As3JsonPath<AS3ValidationError>>>()
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            }
            (AS3Validator::Date, AS3Data::String(items)) => {
                // Ok(items.iter().all(|item| items_type.check(item)))
                let date_regex =
                    Regex::new(r"^\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$").unwrap();

                if !date_regex.is_match(items) {
                    return Err(As3JsonPath(
                        path.to_string(),
                        AS3ValidationError::Generic(format!(
                            " `{}` can't be converted to a valid date. [Supported YYYY-MM-DD] ",
                            items
                        )),
                    ));
                };
                Ok(())
            }
            (AS3Validator::Boolean, AS3Data::Boolean(..)) => Ok(()),

            _ => Err(As3JsonPath(
                path.to_string(),
                AS3ValidationError::TypeError {
                    expected: self.clone(),
                    got: data.clone(),
                },
            )),
        }
    }

    fn check_map_key_value(
        key: &String,
        wanted_type: &AS3Validator,
        path: &mut String,
    ) -> Result<(), String> {
        let _ = match wanted_type {
            AS3Validator::String { .. } => wanted_type.check(&AS3Data::String(key.clone()), path),
            AS3Validator::Integer { .. } => {
                let Ok(n) = key.clone().parse::<i64>() else {
                    return Err(format!("The Key `{}` can't be converted to an Integer", key));
                };

                match wanted_type.check(&&AS3Data::Integer(n), path) {
                    Ok(()) => Ok(()),
                    Err(e) => return Err(e.to_string()),
                }
            }
            AS3Validator::Boolean => match key.to_lowercase().as_str() {
                "true" | "false" | "1" | "0" => Ok(()),
                _ => return Err(format!("The Key `{}` can't be converted to a Boolean", key)),
            },
            AS3Validator::Date => match wanted_type.check(&AS3Data::String(key.clone()), path) {
                Ok(())=> Ok(()),
                _ => return Err(format!("The Key `{}` can't be converted to a Date", key)),
            },
            _ => return Err(
                "Usupported Map's KeyValue conversion. [Supported types : String, Integer, Bool, Date(YYYY-MM-DD) ]"
                    .to_string(),
            ),
        };
        Ok(())
    }
    pub fn to_yaml_string(self) -> String {
        let serialized_json = serde_json::to_string(&self).unwrap();
        let serialized_yaml: serde_yaml::Value =
            serde_yaml::from_str::<serde_yaml::Value>(&serialized_json).unwrap();
        serde_yaml::to_string(&serialized_yaml).unwrap()
    }

    pub fn from(yaml_config: &serde_yaml::Value) -> Result<AS3Validator, String> {
        let serde_yaml::Value::Mapping(inner) = yaml_config else {
            println!("Definition must start with a Yaml Mapping");
            return Err("Definition must start with a Yaml Mapping".to_string());
        };
        let mut root_word: String = "Root".to_string();
        if !inner.contains_key(&root_word) {
            return Err(format!("Missing root word `{root_word}` from definition"));
        };

        AS3Validator::build_from_yaml(&inner.get(&root_word).unwrap(), &mut root_word)
    }

    fn build_from_yaml(
        // validator: &mut AS3Validator,
        yaml_config: &&serde_yaml::Value,
        path: &mut String,
    ) -> Result<AS3Validator, String> {
        // Used to get the validator_type from the canonical long form and also from the shortened syntax
        let validator_type = match (yaml_config.get("+type"), yaml_config) {
            (Some(serde_yaml::Value::String(validator_type)), _) => validator_type,
            (_, serde_yaml::Value::String(validator_type)) => validator_type,
            _ => return Err(format!("Type definition missing for {path} ")),
        };

        let nullable = validator_type.contains("?");

        let validator = match (validator_type.replace("?", "").as_str(), yaml_config) {
            ("Object", serde_yaml::Value::Mapping(inner)) => {
                let x: HashMap<String, AS3Validator> = inner
                    .into_iter()
                    .filter(|(key, _)| key != &&serde_yaml::Value::String("+type".to_string()))
                    .map(|(key, value)| {
                        let mut temp_path = path.clone();
                        temp_path.push_str(" -> ");
                        temp_path.push_str(&key.as_str().unwrap());
                        (
                            key.as_str().unwrap().to_string(),
                            AS3Validator::build_from_yaml(&value, &mut temp_path).unwrap(),
                        )
                    })
                    .collect();

                AS3Validator::Object(x)
            }
            ("String", serde_yaml::Value::Mapping(inner)) => {
                let regex = if let Some(serde_yaml::Value::String(regex)) = inner.get("+regex") {
                    Some(regex.clone())
                } else {
                    None
                };

                let max_length = {
                    let x: Vec<&&str> = ["+MaxLength", "+maxLength", "+max_length"]
                        .iter()
                        .filter(|key_word| inner.get(key_word).is_some())
                        .collect();

                    if x.len() > 1 {
                        return Err(
                            format!("Multiple field indicating the maximum length of a String have been passed : ({})", x.iter().map(|k|k.to_string()).collect::<Vec<String>>().join(",") ),
                        );
                    }

                    if let Some(key_word) = x.first() {
                        if let Some(serde_yaml::Value::Number(max_length)) = inner.get(key_word) {
                            if let Some(max_length) = max_length.as_i64() {
                                Some(max_length)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };

                let min_length = {
                    let x: Vec<&&str> = ["+MinLength", "+minLength", "+min_length"]
                        .iter()
                        .filter(|key_word| inner.get(key_word).is_some())
                        .collect();

                    if x.len() > 1 {
                        return Err(
                                format!("Multiple field indicating the minimum length of a String have been passed : ({})", x.iter().map(|k|k.to_string()).collect::<Vec<String>>().join(",") ),
                            );
                    }
                    if let Some(key_word) = x.first() {
                        if let Some(serde_yaml::Value::Number(max_length)) = inner.get(key_word) {
                            if let Some(max_length) = max_length.as_i64() {
                                Some(max_length)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };

                AS3Validator::String {
                    regex,
                    max_length,
                    min_length,
                }
            }
            ("Date", serde_yaml::Value::Mapping(..)) => AS3Validator::Date,

            ("Integer", serde_yaml::Value::Mapping(inner)) => {
                let maximum = if let Some(serde_yaml::Value::Number(max_length)) = inner.get("+max")
                {
                    if let Some(max_length) = max_length.as_i64() {
                        Some(max_length)
                    } else {
                        None
                    }
                } else {
                    None
                };

                let minimum = if let Some(serde_yaml::Value::Number(max_length)) = inner.get("+min")
                {
                    if let Some(max_length) = max_length.as_i64() {
                        Some(max_length)
                    } else {
                        None
                    }
                } else {
                    None
                };

                AS3Validator::Integer { minimum, maximum }
            }
            ("Decimal" | "Float", serde_yaml::Value::Mapping(inner)) => {
                let maximum = if let Some(serde_yaml::Value::Number(max_length)) = inner.get("+max")
                {
                    if let Some(max_length) = max_length.as_f64() {
                        Some(max_length)
                    } else {
                        None
                    }
                } else {
                    None
                };

                let minimum = if let Some(serde_yaml::Value::Number(max_length)) = inner.get("+min")
                {
                    if let Some(max_length) = max_length.as_f64() {
                        Some(max_length)
                    } else {
                        None
                    }
                } else {
                    None
                };

                AS3Validator::Decimal { minimum, maximum }
            }
            ("List", serde_yaml::Value::Mapping(..)) => {
                let Some(value_type) = yaml_config.get("+ValueType") else {
                    return Err("List defined without the required `+ValueType` property".to_string());
                };
                let list_value_type = AS3Validator::build_from_yaml(&value_type, path).unwrap();

                AS3Validator::List(Box::new(list_value_type))
            }
            ("Map", serde_yaml::Value::Mapping(..)) => {
                let (Some(key_type), Some(value_type)) = (yaml_config.get("+KeyType"), yaml_config.get("+ValueType")) else {
                    return Err(format!("Map MUST have the `+KeyType` and `+ValueType` fields [ {} ] ", path));
                };

                AS3Validator::Map {
                    key_type: Box::new(
                        match AS3Validator::build_from_yaml(
                            &key_type,
                            &mut format!("{} -> +KeyType", path),
                        ) {
                            Ok(d) => d,
                            Err(e) => return Err(e),
                        },
                    ),

                    value_type: Box::new(
                        match AS3Validator::build_from_yaml(
                            &value_type,
                            &mut format!("{} -> +KeyType", path),
                        ) {
                            Ok(d) => d,
                            Err(e) => return Err(e),
                        },
                    ),
                }
            }
            ("Bool" | "Boolean", serde_yaml::Value::Mapping(..)) => AS3Validator::Boolean,

            // Responsable for the abbreviated syntax
            (type_def, serde_yaml::Value::String(..)) => match type_def {
                "String" => AS3Validator::String {
                    regex: None,
                    max_length: None,
                    min_length: None,
                },
                "Integer" => AS3Validator::Integer {
                    minimum: None,
                    maximum: None,
                },
                "Decimal" => AS3Validator::Decimal {
                    minimum: None,
                    maximum: None,
                },
                "Date" => AS3Validator::Date,
                "Bool" => AS3Validator::Boolean,
                _ => {
                    return Err(format!(
                        " {validator_type} can't be used without the `+type` property"
                    ))
                }
            },
            _ => return Err(format!(" {validator_type} is an unsupported type")),
        };

        if nullable {
            Ok(AS3Validator::Nullable(Box::new(validator)))
        } else {
            Ok(validator)
        }
    }
}
