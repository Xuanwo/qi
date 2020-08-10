use super::*;
use std::error::Error;
use std::fs::File;

#[test]
fn json_deserialize() -> Result<(), Box<dyn Error>> {
    let file = File::open("tests/petstore.json")?;

    let spec = v3::from_json_reader(file)?;

    print!("{}", format!("{:?}", spec));

    Ok(())
}

#[test]
fn yaml_deserialize() -> Result<(), Box<dyn Error>> {
    let file = File::open("tests/s3.yaml")?;

    let spec = v3::from_yaml_reader(file)?;
    print!("{}", format!("{:?}", spec));

    Ok(())
}
