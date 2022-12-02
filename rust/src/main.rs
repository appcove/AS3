use clap::{Arg, Parser};
use std::{fs, path::PathBuf};

use as3::{validator::AS3Validator, AS3Data};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None,propagate_version = true)]
struct Args {
    #[clap(long, help = "File with definition")]
    // #[arg(value_parser = clap::value_parser!(PathBuf))]
    #[arg(value_parser = check_file_path)]
    definition: PathBuf,
    #[clap(long, help = "File with the data to verify")]
    #[arg(value_parser = check_file_path)]
    input: PathBuf,
}

fn check_file_path(path: &str) -> Result<PathBuf, String> {
    let path = std::path::Path::new(&path).to_path_buf();
    match (path.exists(), path.is_file()) {
        (true, true) => Ok(path),
        (_, false) => Err(format!(
            "The specified path {path:?} is a folder and not a file"
        )),
        _ => Err(format!("The specified path {path:?} doesn't exists")),
    }
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let Ok(definition) =   serde_yaml::from_str::<serde_yaml::Value>(&std::fs::read_to_string(&args.definition).unwrap()) else {
        return Err(format!("error: The definition file {:?} is not propper json or yaml", &args.definition))
    };

    // let data = match &args.input {
    //     Some(path) => {
    //         let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(path).unwrap())else {
    //             return Err(format!("error: The Data file {:?} is not propper json or yaml", &args.definition))
    //         } ;
    //         json_data
    //     }
    //     None => {
    //         // let x = String::from_iter(std::io::stdin().lines().into_iter();
    //         // std::io::stdin().lines();

    //         // let input = std::io::stdin()
    //         //     .lines()
    //         //     .fold("".to_string(), |acc, line| acc + &line.unwrap() + "\n");

    //         match serde_json::from_str::<serde_json::Value>(&input) {
    //             Ok(json_data) => json_data,
    //             Err(e) => return Err(format!("Could not serialise the piped data : {e}")),
    //         }
    //         // let stdin = std::io::stdin();
    //         // for line in stdin.lines() {
    //         //     let line = line.expect("Could not read line from standard in");
    //         //     if line.is_empty() {
    //         //         return Err("Data has not been passed. Use `--input <data_path>` or pipe it `cat data.json | as3 --definition <definition_path>`".to_owned());
    //         //     }
    //         //     println!("{}", line);
    //         // }
    //         unimplemented!()
    //     }
    // };

    let Ok(data) =  serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(&args.input).unwrap()) else {
        return Err(format!("error: The Data file {:?} is not propper json or yaml", &args.input))
    };

    let validator = AS3Validator::from(&definition).unwrap();
    match validator.validate(&AS3Data::from(&data)) {
        Ok(_) => println!("✅✅ The provided schema matches the data"),
        Err(e) => return Err(format!("❌❌ {}", e.to_string())),
    }
    Ok(())
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Args::command().debug_assert()
}
