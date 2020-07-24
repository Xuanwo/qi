pub mod v3;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::path::Path;

    #[test]
    fn json_deserialize() {
        let path = Path::new("/tmp/openapi.json");
        let file = match File::open(&path) {
            Err(why) => panic!("open file: {}", why.to_string()),
            Ok(file) => file,
        };

        let spec = v3::from_json_reader(file);

        print!("{}", format!("{:?}", spec));
    }
}
