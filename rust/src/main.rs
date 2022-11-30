use std::fs;

use as3::{validator::AS3Validator, AS3Data};

fn main() {
    let data = fs::read_to_string("test.json").expect("Unable to read file");
    let data_to_validate: serde_json::Value =
        serde_json::from_str(&data).expect("JSON does not have correct format.");

    let validator_schema = fs::read_to_string("validator_schema.yml").expect("Unable to read file");
    let schema_yaml: serde_yaml::Value = serde_yaml::from_str(&validator_schema).unwrap();
    println!("{:#?}", schema_yaml);
    if let Ok(validator) = AS3Validator::from(&schema_yaml) {
        println!(
            "{:?}",
            validator.validate(&AS3Data::from(&data_to_validate))
        );
        println!("{:#}", validator.to_yaml_string());
    }
}
