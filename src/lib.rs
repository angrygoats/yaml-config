// TODO: TOMORROW - FINISH FOR DEMO THURS.
//  [ ] - Add tests with simulated environment variables.
//  [ ] - Add tests to insure that when env exists and key exists it prefers env.
//  [ ] - Add tests for when key exists but env does not exist, but prefer env is set it takes the key.
//  [ ] - Improve comments.
//  [ ] - Move tests to own file somewhere look for examples.
//  [ ] -  https://stackoverflow.com/questions/25530035/how-to-structure-large-number-of-unit-tests
//  [ ] -    ---> Note the extern crate comment find examples.
//  [ ] - Run tests make sure everything is kosher. Deploy.
pub mod error;

use crate::error::ParseError;
use enum_as_inner::EnumAsInner;
use fxhash::FxBuildHasher;
use indexmap::IndexMap;
use linked_hash_map::LinkedHashMap;
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
/// # Examples
///
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
            match env_or_error(key) {
                Ok(v) => {
                    map.insert(key.to_string(), Value::String(v));
                }
                Err(_) => {
                    map.insert(
                        key.to_string(),
                        Value::String(maybe_val.as_str().unwrap().to_string()),
                    );
                }
            };
        } else {
            map.insert(
                key.to_string(),
                Value::String(maybe_val.as_str().unwrap().to_string()),
            );
        }

        return Ok(());
    }

    if maybe_val.as_i64().is_some() {
        if prefer_env {
            match env_or_error(key) {
                Ok(v) => {
                    let e_val = v.parse::<i64>().unwrap();
                    map.insert(key.to_string(), Value::I64(e_val));
                }
                Err(_) => {
                    map.insert(key.to_string(), Value::I64(maybe_val.as_i64().unwrap()));
                }
            };
        } else {
            map.insert(key.to_string(), Value::I64(maybe_val.as_i64().unwrap()));
        }

        return Ok(());
    }

    if maybe_val.as_bool().is_some() {
        if prefer_env {
            match env_or_error(key) {
                Ok(v) => {
                    let e_val = v.parse::<bool>().unwrap();
                    map.insert(key.to_string(), Value::Bool(e_val));
                }
                Err(_) => {
                    map.insert(key.to_string(), Value::Bool(maybe_val.as_bool().unwrap()));
                }
            };
        } else {
            map.insert(key.to_string(), Value::Bool(maybe_val.as_bool().unwrap()));
        }

        return Ok(());
    }

    if maybe_val.as_f64().is_some() {
        if prefer_env {
            match env_or_error(key) {
                Ok(v) => {
                    let e_val = v.parse::<f64>().unwrap();
                    map.insert(key.to_string(), Value::F64(e_val));
                }
                Err(_) => {
                    map.insert(key.to_string(), Value::F64(maybe_val.as_f64().unwrap()));
                }
            };
        } else {
            map.insert(key.to_string(), Value::F64(maybe_val.as_f64().unwrap()));
        }

        Ok(())
    } else {
        let msg = format!("Failed to convert type for {}", key);
        Err(ParseError {
            module: "config".to_string(),
            message: msg,
        })
    }
}

/// Converts a YAML key into a string for processing.
fn key_string(key: &Yaml) -> Result<&str, ParseError> {
    match key.as_str() {
        Some(s) => Ok(s),
        None => {
            return Err(ParseError {
                module: "config".to_string(),
                message: format!("Could not convert key {:?} into String.", key),
            })
        }
    }
}

/// Recursive map builder.
///
/// Given a "root" of the yaml file it will generate a configuration recursively. Due
/// to it's use of recursion the actual depth of the YAML file is limited to the depth of
/// the stack. But given most (arguably 99.9%) of YAML files are not even 5 levels deep
/// this seemed like an acceptable trade off for an easier to write algorithm.
///
/// Effectively, this performs a depth first search of the YAML file treating each top level
/// feature as a tree with 1-to-N values. When a concrete (non-hash) value is arrived at
/// the builder constructs a depth-based string definining it.
///
/// The arguments enforce an `FxBuildHasher` based `IndexMap` to insure extremely fast
/// searching of the map. *this map is modified in place*.
///
/// # Arguments
///
/// * `root` - The start of the YAML document as given by `yaml-rust`.
/// * `config` - An IndexMap of String -> Value. It must use an FxBuilderHasher.
/// * `prefer_env` - When `true` will return an environment variable matching the path string
///                  regardless of whether the YAML contains a value for this key. It will prefer
///                  the given value otherwise unless that value is `null`.
/// * `current_key_str` - An optional argument that stores the current string of the path.
///
fn build_map(
    root: &LinkedHashMap<Yaml, Yaml>,
    config: &mut IndexMap<String, Value, FxBuildHasher>,
    prefer_env: bool,
    current_key_str: Option<&str>,
) -> Result<(), ParseError> {
    // Recursively parse each root key to resolve.
    for key in root.keys() {
        let maybe_val = &root[key];

        let key_str = match current_key_str {
            Some(k) => {
                // In this case we have a previous value.
                // We need to construct the current depth-related key.
                let mut next_key = k.to_uppercase().to_string();
                next_key.push_str("_");
                next_key.push_str(&key_string(key)?.to_uppercase());
                next_key
            }
            None => key_string(key)?.to_uppercase().to_string(),
        };

        if maybe_val.is_array() {
            return Err(ParseError {
                module: "config::build_map".to_string(),
                message: "Arrays are currently unsupported for configuration.".to_string(),
            });
        }

        if !maybe_val.as_hash().is_some() {
            // Base condition
            maybe_yaml_to_value(&key_str.to_uppercase(), maybe_val, prefer_env, config)?;
        } else {
            // Now we need to construct the key for one layer deeper.
            build_map(
                maybe_val.as_hash().unwrap(),
                config,
                prefer_env,
                Some(&key_str),
            )?;
        }
    }

    Ok(())
}

/// Loads a configuration file.
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
/// The resulting `IndexMap` will have string keys representing the path
/// configuration described above, and values that are contained in the `Value`
/// enum. See the documentation for `config::Value` for more information on
/// usage.
///
/// # Arguments
///
/// * `file_path` - A string representing the path to the YAML file.
/// * `preference` - The preference for handling values when a key has a value in the
///
/// # Examples
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

    build_map(&user_config, &mut config, prefer_env, None)?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::{env_or_error, load, maybe_yaml_to_value, Value};
    use envtestkit::lock::{lock_read, lock_test};
    use envtestkit::set_env;
    use fxhash::{FxBuildHasher, FxHasher};
    use indexmap::IndexMap;
    use std::ffi::OsString;
    use std::fs::File;
    use std::hash::BuildHasherDefault;
    use std::io::Write;
    use tempfile::tempdir;
    use yaml_rust::Yaml;

    #[test]
    fn successfully_gets_environment_variable() {
        let _lock = lock_test();
        let _test = set_env(OsString::from("TEST_ENV_VAR"), "1");
        let res = env_or_error("TEST_ENV_VAR").expect("failed to find environment variable.");
        assert_eq!(res, "1");
    }

    #[test]
    fn error_when_environment_variable_is_not_found() {
        let _lock = lock_read();
        let res = env_or_error("TEST_ENV_VAR");
        assert!(res.is_err());
    }

    #[test]
    fn maybe_yaml_null_gets_environment_variable_i64() {
        // This simulates something that would be mapped by
        // ```
        // test_env:
        //   var: null
        // ```
        let _lock = lock_test();
        let _test = set_env(OsString::from("TEST_ENV_VAR"), "1");
        let mut config: IndexMap<String, Value, BuildHasherDefault<FxHasher>> =
            IndexMap::with_hasher(FxBuildHasher::default());

        let maybe_val = Yaml::from_str("null");

        maybe_yaml_to_value("TEST_ENV_VAR", &maybe_val, false, &mut config).unwrap();

        assert_eq!(*config["TEST_ENV_VAR"].as_i64().unwrap(), 1);
    }

    #[test]
    fn maybe_yaml_null_gets_environment_variable_f64() {
        // This simulates something that would be mapped by
        // ```
        // test_env:
        //   var: null
        // ```
        let _lock = lock_test();
        let _test = set_env(OsString::from("TEST_ENV_VAR"), "3.14");
        let mut config: IndexMap<String, Value, BuildHasherDefault<FxHasher>> =
            IndexMap::with_hasher(FxBuildHasher::default());

        let maybe_val = Yaml::from_str("null");

        maybe_yaml_to_value("TEST_ENV_VAR", &maybe_val, false, &mut config).unwrap();

        assert_eq!(*config["TEST_ENV_VAR"].as_f64().unwrap(), 3.14);
    }

    #[test]
    fn maybe_yaml_null_gets_environment_variable_bool() {
        // This simulates something that would be mapped by
        // ```
        // test_env:
        //   var: null
        // ```
        let _lock = lock_test();
        let _test = set_env(OsString::from("TEST_ENV_VAR"), "true");
        let mut config: IndexMap<String, Value, BuildHasherDefault<FxHasher>> =
            IndexMap::with_hasher(FxBuildHasher::default());

        let maybe_val = Yaml::from_str("null");

        maybe_yaml_to_value("TEST_ENV_VAR", &maybe_val, false, &mut config).unwrap();

        assert_eq!(*config["TEST_ENV_VAR"].as_bool().unwrap(), true);
    }

    #[test]
    fn maybe_yaml_null_gets_environment_variable_string() {
        // This simulates something that would be mapped by
        // ```
        // test_env:
        //   var: null
        // ```
        let _lock = lock_test();
        let _test = set_env(OsString::from("TEST_ENV_VAR"), "string");
        let mut config: IndexMap<String, Value, BuildHasherDefault<FxHasher>> =
            IndexMap::with_hasher(FxBuildHasher::default());

        let maybe_val = Yaml::from_str("null");

        maybe_yaml_to_value("TEST_ENV_VAR", &maybe_val, false, &mut config).unwrap();

        assert_eq!(*config["TEST_ENV_VAR"].as_string().unwrap(), "string");
    }

    #[test]
    fn maybe_yaml_null_gets_environment_variable_string_with_prefer_yaml() {
        // This simulates something that would be mapped by
        // ```
        // test_env:
        //   var: null
        // ```
        let _lock = lock_test();
        let _test = set_env(OsString::from("TEST_ENV_VAR"), "string");
        let mut config: IndexMap<String, Value, BuildHasherDefault<FxHasher>> =
            IndexMap::with_hasher(FxBuildHasher::default());

        let maybe_val = Yaml::from_str("null");

        maybe_yaml_to_value("TEST_ENV_VAR", &maybe_val, true, &mut config).unwrap();

        assert_eq!(*config["TEST_ENV_VAR"].as_string().unwrap(), "string");
    }

    #[test]
    fn maybe_yaml_gets_i64() {}

    #[test]
    fn maybe_yaml_gets_i64_env_var_match() {}

    #[test]
    fn maybe_yaml_gets_f64() {}

    #[test]
    fn maybe_yaml_gets_f64_env_var_match() {}

    #[test]
    fn maybe_yaml_gets_bool() {}

    #[test]
    fn maybe_yaml_gets_bool_env_var_match() {}

    #[test]
    fn maybe_yaml_gets_string() {}

    #[test]
    fn maybe_yaml_gets_string_env_var_match() {}

    #[test]
    fn arrays_are_not_allowed() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.yaml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(
            file,
            "
            test_key_1: 1
            test_key_2: \"test\"
            test_key_3:
                - test_1: 0
                - test_3: 2
                - test_4: 'a'
            test_key_4: true
            ",
        )
        .unwrap();

        let res = load(file_path.to_str().unwrap(), None);

        assert!(res.is_err());

        drop(file);
        dir.close().unwrap();
    }

    #[test]
    fn one_layer() {
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

        let res = load(file_path.to_str().unwrap(), None).expect("temp file not loaded.");

        assert_eq!(*res["TEST_KEY_1"].as_i64().unwrap(), 1);
        assert_eq!(*res["TEST_KEY_2"].as_string().unwrap(), "test");
        assert_eq!(*res["TEST_KEY_3"].as_f64().unwrap(), 3.14);
        assert_eq!(*res["TEST_KEY_4"].as_bool().unwrap(), true);

        drop(file);
        dir.close().unwrap();
    }

    #[test]
    fn two_layer() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.yaml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(
            file,
            "
            test_key_1:
              sub_key_a: 1
              sub_key_b: 2
            test_key_2: \"test\"
            test_key_3:
              sub_key_a: 3.14
              sub_key_b: 6.28
            test_key_4: true
            ",
        )
        .unwrap();

        let res = load(file_path.to_str().unwrap(), None).expect("lol");

        assert_eq!(*res["TEST_KEY_1_SUB_KEY_A"].as_i64().unwrap(), 1);
        assert_eq!(*res["TEST_KEY_1_SUB_KEY_B"].as_i64().unwrap(), 2);
        assert_eq!(*res["TEST_KEY_2"].as_string().unwrap(), "test");
        assert_eq!(*res["TEST_KEY_3_SUB_KEY_A"].as_f64().unwrap(), 3.14);
        assert_eq!(*res["TEST_KEY_3_SUB_KEY_B"].as_f64().unwrap(), 6.28);
        assert_eq!(*res["TEST_KEY_4"].as_bool().unwrap(), true);

        drop(file);
        dir.close().unwrap();
    }

    #[test]
    fn three_layer() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.yaml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(
            file,
            "
            test_key_1:
              sub_key_a:
                sub_sub_key_a: 1
                sub_sub_key_b: 2
              sub_key_b: 2
            test_key_2: \"test\"
            test_key_3:
              sub_key_a: 3.14
              sub_key_b: 6.28
            test_key_4: true
            ",
        )
        .unwrap();

        let res = load(file_path.to_str().unwrap(), None).expect("lol");

        assert_eq!(
            *res["TEST_KEY_1_SUB_KEY_A_SUB_SUB_KEY_A"].as_i64().unwrap(),
            1
        );
        assert_eq!(
            *res["TEST_KEY_1_SUB_KEY_A_SUB_SUB_KEY_B"].as_i64().unwrap(),
            2
        );
        assert_eq!(*res["TEST_KEY_2"].as_string().unwrap(), "test");
        assert_eq!(*res["TEST_KEY_3_SUB_KEY_A"].as_f64().unwrap(), 3.14);
        assert_eq!(*res["TEST_KEY_3_SUB_KEY_B"].as_f64().unwrap(), 6.28);
        assert_eq!(*res["TEST_KEY_4"].as_bool().unwrap(), true);

        drop(file);
        dir.close().unwrap();
    }
}
