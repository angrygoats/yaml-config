// TODO: TOMORROW
//  [ ] - Add test cases for nested/two-three layer nested
//  [ ] - Add nesting code
//  [ ] - Run tests make sure everything is kosher. Deploy.
pub mod error;

use crate::error::ParseError;
use enum_as_inner::EnumAsInner;
use fxhash::FxBuildHasher;
use indexmap::IndexMap;
use std::env;
use std::fs::read_to_string;
use yaml_rust::{Yaml, YamlLoader};

/// Defines the preference for loading of a configuration when a variable exists in the
/// YAML and also along the same path in the environment.
#[derive(Debug, PartialEq, Eq)]
pub enum Preference {
    PreferYaml,
    PreferEnv,
}

/// A wrapped type enum useful for allowing polymorphic returns from
/// the map creation function.
///
/// **Examples**
/// fn main() {
///
/// ```rust
/// use config::Value;
/// let x = Value::I32(10);
/// let val = *x.as_i32().unwrap();
/// ```
/// }
#[derive(Debug, EnumAsInner)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    String(String),
    Bool(bool),
}

/// Provides a simple way to allow question mark syntax in order to
/// convert environment errors into ParseErrors.
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

/// Takes a key and a Yaml reference, parses it, and sets the key.
///
/// In addition to doing the initial parsing it will also do environment finding. If a given
/// key is null, or `prefer_env` is true, then it will search the environment for the given
/// key string and attempt to use that key string's value.
///
fn maybe_yaml_to_value(
    key: &str,
    maybe_val: &Yaml,
    prefer_env: bool,
    map: &mut IndexMap<String, Value, FxBuildHasher>,
) -> Result<(), ParseError> {
    if maybe_val.is_null() {
        // Because the value is null we have to attempt a full parse of whatever is coming back
        // from the user's environment since we don't have an indicator from the YAML itself.
        let val_str = env_or_error(key)?;

        let val = match val_str.parse::<i64>() {
            Ok(v) => Value::I64(v),
            Err(_) => match val_str.parse::<f64>() {
                Ok(v) => Value::F64(v),
                Err(_) => match val_str.parse::<bool>() {
                    Ok(v) => Value::Bool(v),
                    Err(_) => Value::String(val_str),
                },
            },
        };

        map.insert(key.to_string(), val);
        return Ok(());
    }

    if maybe_val.as_str().is_some() {
        if prefer_env {
            map.insert(key.to_string(), Value::String(env_or_error(key)?));
        }
        let val = maybe_val.as_str().unwrap().to_string();
        map.insert(key.to_string(), Value::String(val));
        return Ok(());
    }

    if maybe_val.as_i64().is_some() {
        if prefer_env {
            let val = env_or_error(key)?;
            let e_val = val.parse::<i64>().unwrap();
            map.insert(key.to_string(), Value::I64(e_val));
        }

        map.insert(key.to_string(), Value::I64(maybe_val.as_i64().unwrap()));
        return Ok(());
    }

    if maybe_val.as_bool().is_some() {
        if prefer_env {
            let val = env_or_error(key)?;
            let e_val = val.parse::<bool>().unwrap();
            map.insert(key.to_string(), Value::Bool(e_val));
        }
        map.insert(key.to_string(), Value::Bool(maybe_val.as_bool().unwrap()));
        return Ok(());
    }

    if maybe_val.as_f64().is_some() {
        if prefer_env {
            let val = env_or_error(key)?;
            let e_val = val.parse::<f64>().unwrap();
            map.insert(key.to_string(), Value::F64(e_val));
        }
        map.insert(key.to_string(), Value::F64(maybe_val.as_f64().unwrap()));
        Ok(())
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
/// use config::load;
/// let configuration = load("path/to/yaml/file.yaml", None);
///
/// ```
///
/// Use with preference:
///
/// ```rust
/// use config::Preference;
/// use config::load;
/// let configuration = load("path/to/yaml/file.yaml",
///                          Some(Preference::PreferEnv));
/// ```
pub fn load(
    file_path: &str,
    preference: Option<Preference>,
) -> Result<IndexMap<String, Value, FxBuildHasher>, ParseError> {
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
        maybe_yaml_to_value(&key_str, maybe_val, prefer_env, &mut config)?;
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

        assert_eq!(*res["TEST_KEY_1"].as_i64().unwrap(), 1);
        assert_eq!(*res["TEST_KEY_2"].as_string().unwrap(), "test");
        assert_eq!(*res["TEST_KEY_3"].as_f64().unwrap(), 3.14);
        assert_eq!(*res["TEST_KEY_4"].as_bool().unwrap(), true);

        drop(file);
        dir.close().unwrap();
    }
}
