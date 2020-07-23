use std::error::Error;

#[cfg(feature = "std")]
pub fn from_json_reader<R>(r: R) -> Result<super::schema::Spec, Box<dyn Error>>
    where
        R: std::io::Read
{
    let spec = serde_json::from_reader(r)?;

    Ok(spec)
}
