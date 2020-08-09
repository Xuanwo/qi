use std::error::Error;

pub fn from_json_reader<R>(r: R) -> Result<super::schema::Spec, Box<dyn Error>>
where
    R: std::io::Read,
{
    let spec = serde_json::from_reader(r)?;

    Ok(spec)
}

pub fn from_yaml_reader<R>(r: R) -> Result<super::schema::Spec, Box<dyn Error>>
where
    R: std::io::Read,
{
    let spec = serde_yaml::from_reader(r)?;

    Ok(spec)
}
