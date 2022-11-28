use super::*;
use serde_json::json;

fn verify(
    data: &serde_json::Value,
    validator_config: &serde_yaml::Value,
    expected: Result<(), AS3ValidationError>,
) {
    let data = AS3Data::from(data);
    let validator = AS3Validator::from(&validator_config).unwrap();
    assert_eq!(validator.validate(&data), expected);
}

#[test]
fn should_run() {
    let data = json!({
      "age": 25,
      "children": 5,
      "name": "Dilec",
      "vehicles": {
        "list": [
          { "name": "model3", "maker": "Tesla", "year": 2018 },
          { "name": "Raptor", "maker": "Ford", "year": 2018 }
        ]
      }
    });

    let validator: serde_yaml::Value = serde_yaml::from_str(
        &r#"
        Root:
            +type: Object
            age: 
                +type: Integer
            children: 
                +type: Integer
            name:
                +type: String
                +Regex: "^[A-Z][a-z]"
            vehicles:
                +type: Object
                list: 
                    +type : List
                    +ValueType: 
                        +type : Object
                        name:
                            +type: String
                        maker:
                            +type: String
                        year:
                            +type: Integer
                        "#,
    )
    .unwrap();

    verify(&data, &validator, Ok(()))
}

#[test]
fn with_decimal_error() {
    let data = json!({
      "vehicles": {
        "list": [
          { "name": "Raptor", "maker": "Ford", "year": 20.18 }
        ]
      }
    });

    let validator: serde_yaml::Value = serde_yaml::from_str(
        &r#"
        Root:
            +type: Object
            vehicles:
                +type: Object
                list: 
                    +type : List
                    +ValueType: 
                        +type : Object
                        name:
                            +type: String
                        maker:
                            +type: String
                        year:
                            +type: Integer
                    "#,
    )
    .unwrap();

    verify(
        &data,
        &validator,
        Err(AS3ValidationError::TypeError {
            expected: AS3Validator::Integer {
                minimum: None,
                maximum: None,
            },
            got: AS3Data::Decimal(20.18),
        }),
    );
}
#[test]
fn with_string_error() {
    let data = json!({
      "vehicles": {
        "list": [
          { "name": "model3", "maker": "Tesla", "year": 2018 },
          { "name": "Raptor", "maker": "Ford", "year": "2018" }
        ]
      }
    });

    let validator: serde_yaml::Value = serde_yaml::from_str(
        &r#"
        Root:
            +type: Object
            vehicles:
                +type: Object
                list: 
                    +type : List
                    +ValueType: 
                        +type : Object
                        name:
                            +type: String
                        maker:
                            +type: String
                        year:
                            +type: Integer
                    "#,
    )
    .unwrap();

    verify(
        &data,
        &validator,
        Err(AS3ValidationError::TypeError {
            expected: AS3Validator::Integer {
                minimum: None,
                maximum: None,
            },
            got: AS3Data::String("2018".to_string()),
        }),
    );
}

#[test]
fn with_regex_error() {
    let data = json!({
      "vehicles": {
        "list": [
          { "name": "model3", "maker": "Tesla", "year": 2018},
          { "name": "Raptor", "maker": "ford", "year": 2018 }
        ]
      }
    });

    let validator: serde_yaml::Value = serde_yaml::from_str(
        &r#"
        Root:
            +type: Object
            vehicles:
                +type: Object
                list: 
                    +type : List
                    +ValueType: 
                        +type : Object
                        name:
                            +type: String
                        maker:
                            +type: String
                            +regex: "^[A-Z][a-z]"
                        year:
                            +type: Integer
                    "#,
    )
    .unwrap();
    verify(
        &data,
        &validator,
        Err(AS3ValidationError::RegexError {
            word: "ford".to_string(),
            regex: "^[A-Z][a-z]".to_string(),
        }),
    );
}

#[test]
fn with_minimum_error() {
    let json = json!({
      "age": 18,
      "children": 5,
    });

    let validator = AS3Validator::Object(HashMap::from([
        (
            "age".to_owned(),
            AS3Validator::Integer {
                minimum: Some(20),
                maximum: None,
            },
        ),
        (
            "children".to_owned(),
            AS3Validator::Integer {
                minimum: Some(2),
                maximum: None,
            },
        ),
    ]));

    assert_eq!(
        validator.validate(&AS3Data::from(&json)),
        Err(AS3ValidationError::Minimum {
            number: 18.0,
            minimum: 20.0
        })
    );

    let json = json!({
      "age": 20,
      "children": 0,
    });

    assert_eq!(
        validator.validate(&AS3Data::from(&json)),
        Err(AS3ValidationError::Minimum {
            number: 0.0,
            minimum: 2.0
        })
    );

    let json = json!({
      "age": 20,
      "children": 20,
    });

    assert_eq!(validator.validate(&AS3Data::from(&json)), Ok(()))
}

#[test]
fn with_missing_field_error_validator_derive() {
    let mut data = json!({
      "vehicles": {
        "name": "raptor",
        "year": 2018
      },
      "Truks": {
        "name": "hummer",
        "maker": "ford",
        "year": 2019
      }
    });

    let validator: serde_yaml::Value = serde_yaml::from_str(
        &r#"
        Root:
            +type: Object
            vehicles:
                +type: Object
                name:
                    +type: String
                maker:
                    +type: String
                year:
                    +type: Integer
            Truks:
                +type: Object
                name:
                    +type: String
                maker:
                    +type: String
                year:
                    +type: Integer
                    "#,
    )
    .unwrap();

    verify(
        &data,
        &validator,
        Err(AS3ValidationError::MissingKey {
            key: "maker".to_string(),
        }),
    );

    data["vehicles"]["maker"] = serde_json::Value::String("tesla".to_string());

    verify(&data, &validator, Ok(()));
}

#[test]
fn with_list() {
    let data = json!(
    {
        "students": [
          {
            "surname": "Bob",
            "year": 2020,
            "grade": "A"
          },
          {
            "surname": "Bob",
            "gg": 2020,
            "grade": "A"
          }
        ]
      });

    let validator: serde_yaml::Value = serde_yaml::from_str(
        &r#"
            Root:
                +type: Object
                students:
                  +type: List
                  +ValueType:
                    +type: Object
                    surname:
                      +type: String
                    year:
                      +type: Integer
                    grade:
                      +type: String
                        "#,
    )
    .unwrap();

    verify(
        &data,
        &validator,
        Err(AS3ValidationError::MissingKey {
            key: "year".to_string(),
        }),
    );

    let data2 = json!(
    {
        "students": [
          {
            "surname": "Bob",
            "year": 2020,
            "grade": 20
          }
        ]
      });

    verify(
        &data2,
        &validator,
        Err(AS3ValidationError::TypeError {
            expected: AS3Validator::String {
                regex: None,
                max_length: None,
                min_length: None,
            },
            got: AS3Data::Integer(20),
        }),
    );
}

#[test]
fn with_map() {
    let data = json!(
    {
        "People": {
          "NY": {
            "name": "casei neistat",
            "age": 48
          },
          "LA": {
            "name": "odhfeo",
            "age": 48
          }
        }
      });

    let validator: serde_yaml::Value = serde_yaml::from_str(
        &r#"
        Root:
            +type: Object
            People:
                +type: Map
                +KeyType:
                    +type: String
                +ValueType:
                    +type: Object
                    name:
                        +type: String
                    age:
                        +type: Integer
                    "#,
    )
    .unwrap();

    verify(&data, &validator, Ok(()));
}

#[test]
fn with_bool_and_map() {
    let data = json!(
    {
          "false": {
            "name": "odhfeo",
            "age": true
          }

      });

    let validator: serde_yaml::Value = serde_yaml::from_str(
        &r#"
        Root:
            +type: Map
            +KeyType:
                +type: Bool
            +ValueType:
                +type: Object
                name:
                    +type: String
                age:
                    +type: Bool
                    "#,
    )
    .unwrap();
    verify(&data, &validator, Ok(()));
}

#[test]
fn with_date_and_map() {
    let data = json!(
    {
          "2020-10-15": {
            "name": "odhfeo",
            "age": "2020-10-15"
          }

      });

    let validator: serde_yaml::Value = serde_yaml::from_str(
        &r#"
        Root:
            +type: Map
            +KeyType:
                +type: Date
            +ValueType:
                +type: Object
                name:
                    +type: String
                age:
                    +type: Date
                    "#,
    )
    .unwrap();

    verify(&data, &validator, Ok(()));

    let data = json!(
    {
          "2020/10/15": {
            "name": "odhfeo",
            "age": "2020-10-15"
          }

      });

    let validator_config: serde_yaml::Value = serde_yaml::from_str(
        &r#"
            Root:
                +type: Map
                +KeyType:
                    +type: Date
                +ValueType:
                    +type: Object
                    name:
                        +type: String
                    age:
                        +type: Date
                        "#,
    )
    .unwrap();

    verify(
        &data,
        &validator_config,
        Err(AS3ValidationError::Generic(
            "The Key `2020/10/15` can't be converted to a Date".to_string(),
        )),
    );
}

#[test]
fn with_abbreviation_types() {
    let data = json!(
    {
        "name": "Dilec",
        "birth": "2022-04-01",
        "age": 21,
        "height" : 1.75,
        "male" : true
    }
    );

    let validator: serde_yaml::Value = serde_yaml::from_str(
        &r#"
        Root:
            +type: Object
            name: String
            age: Integer
            birth : Date
            height : Decimal
            male : Bool
                    "#,
    )
    .unwrap();

    verify(&data, &validator, Ok(()));

    // let data = json!(
    // {

    //         "name": "odhfeo",
    //         "age": "2020-10-15"

    //   });

    // let validator_config: serde_yaml::Value = serde_yaml::from_str(
    //     &r#"
    //         Root:
    //             +type: Map
    //             +KeyType:
    //                 +type: Date
    //             +ValueType:
    //                 +type: Object
    //                 name:
    //                     +type: String
    //                 age:
    //                     +type: Date
    //                     "#,
    // )
    // .unwrap();

    // verify(
    //     &data,
    //     &validator_config,
    //     Err(AS3ValidationError::Generic(
    //         "The Key `2020/10/15` can't be converted to a Date".to_string(),
    //     )),
    // );
}
