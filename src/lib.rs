// TODO: TOMORROW
//  [ ] - Fix crate importing
//  [ ] - Fix comment error checking???
//  [ ] - Add test cases for nested/two-three layer nested
//  [ ] - Add nesting code
//  [ ] - Run tests make sure everything is kosher. Deploy.

use crate::error::ParseError;
use fxhash::FxBuildHasher;
use indexmap::IndexMap;
use std::env;
use std::fs::read_to_string;
use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug, PartialEq, Eq)]
pub enum Preference {
    PreferYaml,
    PreferEnv,
}

/// Provides a simple way to allow question mark syntax in order to
/// convert environment errors into ParseErrors.
///
/// **Examples**
///
/// ```rust
/// let my_var = env_or_error("TEST_ENVIRONMENT_VARIABLE")?;
/// ```
fn env_or_error(key: &str) -> Result<String, ParseError> {
    match env::var_os(key) {
        Some(v) => Ok(v
            .into_string()
            .expect("Could not convert OsString into string.")),
        None => {
            let msg = format!("Error parsing OS environment variable for {}", key);
            Err(ParseError {
                module: "std::env".to_string(),
                message: msg,
            })
        }
    }
}

fn maybe_yaml_to_string(
    key: &str,
    maybe_val: &Yaml,
    prefer_env: bool,
) -> Result<String, ParseError> {
    if prefer_env {
        return env_or_error(key);
    }

    if maybe_val.is_null() {
        return env_or_error(key);
    }

    if maybe_val.as_str().is_some() {
        return Ok(maybe_val.as_str().unwrap().to_string());
    }

    if maybe_val.as_i64().is_some() {
        return Ok(maybe_val.as_i64().unwrap().to_string());
    }

    if maybe_val.as_bool().is_some() {
        return Ok(maybe_val.as_bool().unwrap().to_string());
    }

    if maybe_val.as_f64().is_some() {
        return Ok(maybe_val.as_f64().unwrap().to_string());
    } else {
        let msg = format!("Failed to convert type for {}", key);
        Err(ParseError {
            module: "config".to_string(),
            message: msg,
        })
    }
}

/// Given a valid path, loads a configuration file.
///
/// The parser will first load the YAML file. It then re-organizes the YAML
/// file into a common naming convention. Given:
///
/// ```yaml
/// X:
///   y: "value"
/// ```
///
/// The key will be `X_Y` and the value will be the string `"value"`.
///
/// After loading, it investigates each value looking for nulls. In the
/// case of a null, it will search the environment for the
/// key (in the above example `X_Y`). If found, it replaces the value.
/// If not found, it will error.
///
/// In the event that a key in the environment matches a key that is
/// provided in the YAML it will prefer the key in the YAML file. To
/// override this, pass a `Some(Preference::PreferEnv)` to the
/// `preference` argument.
///
/// **Examples**
///
/// ```rust
/// use config;
/// let configuration = config::load("path/to/yaml/file.yaml", None)
///                         .expect("Failed to load file!");
/// ```
///
/// Use with preference:
///
/// ```rust
/// use config;
/// let configuration = config::load(
///                             "path/to/yaml/file.yaml",
///                             Some(config::Preference::PreferEnv)
///                         ).expect("Failed to load file!");
/// ```
pub fn load(
    file_path: &str,
    preference: Option<Preference>,
) -> Result<IndexMap<String, String, FxBuildHasher>, ParseError> {
    let prefer_env = match preference {
        Some(p) => p == Preference::PreferEnv,
        None => false,
    };
    let doc_str = read_to_string(file_path)?;
    let yaml_docs = YamlLoader::load_from_str(&doc_str)?;
    let base_config = &yaml_docs[0];
    let user_config = match base_config.as_hash() {
        Some(hash) => hash,
        None => {
            return Err(ParseError {
                module: "config".to_string(),
                message: "Failed to parse YAML as hashmap.".to_string(),
            })
        }
    };

    let mut config = IndexMap::with_hasher(FxBuildHasher::default());

    for key in user_config.keys() {
        let maybe_val = &user_config[key];

        let mut key_str = match key.as_str() {
            Some(s) => s.to_string().to_uppercase(),
            None => {
                return Err(ParseError {
                    module: "config".to_string(),
                    message: format!("Could not convert key {:?} into String.", key),
                })
            }
        };

        // TODO:
        // We can match maybe_val as hash() to do this.
        // Just use a while loop to go as deep as possible appending to key_str each time.
        // We need to check if val type is another hash. If it is we have subkey and need
        // to continue processing until we have the entire path from root -> value. Val
        // should be pointing at an actual static value by the time it gets to the below.

        // TODO:
        // Match and convert to string. At a later time we can try doing after-the-fact conversion
        // based on known maybe val types to go from string -> concrete type. It cannot be done
        // in one function due to generic parameter restrictions.
        let val = maybe_yaml_to_string(&key_str, maybe_val, prefer_env)?;

        // TODO: Here we can have a parse-try block with the various types.

        config.insert(key_str, val);
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::load;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn basic_one_layer() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.yaml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(
            file,
            "
            test_key_1: 1
            test_key_2: \"test\"
            test_key_3: 3.14
            test_key_4: true
            ",
        )
        .unwrap();

        let res = load(file_path.to_str().unwrap(), None).expect("lol");

        assert_eq!(res["TEST_KEY_1"], "1");
        assert_eq!(res["TEST_KEY_2"], "test");
        assert_eq!(res["TEST_KEY_3"], "3.14");
        assert_eq!(res["TEST_KEY_4"], "true");

        drop(file);
        dir.close().unwrap();
    }
}
