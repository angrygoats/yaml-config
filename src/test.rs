mod test {
    use crate::{env_or_error, load, maybe_yaml_to_value, Value};
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
    fn maybe_yaml_gets_i64() {
        // This simulates something that would be mapped by
        // ```
        // test_var:
        //   val: 10
        // ```
        let mut config: IndexMap<String, Value, BuildHasherDefault<FxHasher>> =
            IndexMap::with_hasher(FxBuildHasher::default());

        let maybe_val = Yaml::Integer(10);

        maybe_yaml_to_value("TEST_VAR_VAL", &maybe_val, true, &mut config).unwrap();

        assert_eq!(*config["TEST_VAR_VAL"].as_i64().unwrap(), 10);
    }

    #[test]
    fn maybe_yaml_gets_i64_env_var_match() {
        // This simulates something that would be mapped by
        // ```
        // test_var:
        //   val: 10
        // ```
        // When an environment variable matches and PreferYaml is given.
        let _lock = lock_test();
        let _test = set_env(OsString::from("TEST_ENV_VAL"), "15");

        let mut config: IndexMap<String, Value, BuildHasherDefault<FxHasher>> =
            IndexMap::with_hasher(FxBuildHasher::default());

        let maybe_val = Yaml::Integer(10);

        maybe_yaml_to_value("TEST_VAR_VAL", &maybe_val, true, &mut config).unwrap();

        assert_eq!(*config["TEST_VAR_VAL"].as_i64().unwrap(), 10);
    }

    #[test]
    fn maybe_yaml_gets_f64() {
        // This simulates something that would be mapped by
        // ```
        // test_var:
        //   val: 3.14
        // ```
        let mut config: IndexMap<String, Value, BuildHasherDefault<FxHasher>> =
            IndexMap::with_hasher(FxBuildHasher::default());

        let maybe_val = Yaml::from_str("3.14");

        maybe_yaml_to_value("TEST_VAR_VAL", &maybe_val, true, &mut config).unwrap();

        assert_eq!(*config["TEST_VAR_VAL"].as_f64().unwrap(), 3.14);
    }

    #[test]
    fn maybe_yaml_gets_f64_env_var_match() {
        // This simulates something that would be mapped by
        // ```
        // test_var:
        //   val: 3.14
        // ```
        // When an environment variable matches and PreferYaml is given.
        let _lock = lock_test();
        let _test = set_env(OsString::from("TEST_ENV_VAL"), "6.28");

        let mut config: IndexMap<String, Value, BuildHasherDefault<FxHasher>> =
            IndexMap::with_hasher(FxBuildHasher::default());

        let maybe_val = Yaml::from_str("3.14");

        maybe_yaml_to_value("TEST_VAR_VAL", &maybe_val, true, &mut config).unwrap();

        assert_eq!(*config["TEST_VAR_VAL"].as_f64().unwrap(), 3.14);
    }

    #[test]
    fn maybe_yaml_gets_bool() {
        // This simulates something that would be mapped by
        // ```
        // test_var:
        //   val: true
        // ```
        let mut config: IndexMap<String, Value, BuildHasherDefault<FxHasher>> =
            IndexMap::with_hasher(FxBuildHasher::default());

        let maybe_val = Yaml::Boolean(true);

        maybe_yaml_to_value("TEST_VAR_VAL", &maybe_val, true, &mut config).unwrap();

        assert_eq!(*config["TEST_VAR_VAL"].as_bool().unwrap(), true);
    }

    #[test]
    fn maybe_yaml_gets_bool_env_var_match() {
        // This simulates something that would be mapped by
        // ```
        // test_var:
        //   val: true
        // ```
        // When an environment variable matches and PreferYaml is given.
        let _lock = lock_test();
        let _test = set_env(OsString::from("TEST_ENV_VAL"), "true");

        let mut config: IndexMap<String, Value, BuildHasherDefault<FxHasher>> =
            IndexMap::with_hasher(FxBuildHasher::default());

        let maybe_val = Yaml::Boolean(true);

        maybe_yaml_to_value("TEST_VAR_VAL", &maybe_val, true, &mut config).unwrap();

        assert_eq!(*config["TEST_VAR_VAL"].as_bool().unwrap(), true);
    }

    #[test]
    fn maybe_yaml_gets_string() {
        // This simulates something that would be mapped by
        // ```
        // test_var:
        //   val: "test"
        // ```
        let mut config: IndexMap<String, Value, BuildHasherDefault<FxHasher>> =
            IndexMap::with_hasher(FxBuildHasher::default());

        let maybe_val = Yaml::String("test".to_string());

        maybe_yaml_to_value("TEST_VAR_VAL", &maybe_val, true, &mut config).unwrap();

        assert_eq!(*config["TEST_VAR_VAL"].as_string().unwrap(), "test");
    }

    #[test]
    fn maybe_yaml_gets_string_env_var_match() {
        // This simulates something that would be mapped by
        // ```
        // test_var:
        //   val: "test"
        // ```
        // When an environment variable matches and PreferYaml is given.
        let _lock = lock_test();
        let _test = set_env(OsString::from("TEST_ENV_VAL"), "test");

        let mut config: IndexMap<String, Value, BuildHasherDefault<FxHasher>> =
            IndexMap::with_hasher(FxBuildHasher::default());

        let maybe_val = Yaml::from_str("test");

        maybe_yaml_to_value("TEST_VAR_VAL", &maybe_val, true, &mut config).unwrap();

        assert_eq!(*config["TEST_VAR_VAL"].as_string().unwrap(), "test");
    }

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
