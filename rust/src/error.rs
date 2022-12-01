use crate::{validator::AS3Validator, AS3Data};
use thiserror::Error;
#[derive(Error, Debug, PartialEq)]
#[error("{1} in [{0}]. ")]
pub struct As3JsonPath<T: std::error::Error>(pub String, pub T);

#[derive(Error, Debug, PartialEq)]
pub enum AS3ValidationError {
    #[error("Mismatched types. Expected `{:?}` got `{:?}` ." , .expected , .got)]
    TypeError {
        expected: AS3Validator,
        got: AS3Data,
    },
    #[error("Key {} is not in " , .key )]
    MissingKey { key: String },
    #[error("Word {} is not following the `{}` regex  ." , .word, .regex )]
    RegexError { word: String, regex: String },

    #[error(" `{}` is under the minumum of `{}`  ." , .number , .minimum)]
    Minimum { number: f64, minimum: f64 },
    #[error(" `{}` is above the maximum of `{}` ." , .number , .maximum)]
    Maximum { number: f64, maximum: f64 },
    #[error(" Error during validation: {0} ")]
    Generic(String),
    #[error(" {} is {} charcters long, above the max lenght allowed of {} ." , .string, .current_lenght , .max_length)]
    MaximumString {
        string: String,
        current_lenght: i64,
        max_length: i64,
    },

    #[error(" {} is {} charcters long, above the max lenght allowed of {}." , .string, .current_lenght , .min_length)]
    MinimumString {
        string: String,
        current_lenght: i64,
        min_length: i64,
    },

    #[error("field not set as not nullable but is a null")]
    NotNullableNull,
}
