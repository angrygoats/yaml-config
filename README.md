# Dynamic Configurator for Rust

The dynamic configuration library is simple: it takes an arbitrary YAML file, parses it into a
common format, and returns a hashmap of configurations and their keys. It supports environment
loading as well.

Take for example the following:

```yaml
database:
  name: null
  username: null
  password: null
  port: null
logging:
  level: "INFO"
performance:
  threads: 8
```

The YAML file will be parsed into a hashmap using the following rules:

1. Keys are recursively named according to hierarchy. In the example above, one key would be `DATABASE_USERNAME`.
2. If a key is `null` the environment will be searched using the hierarchy name described in (1).
3. If a key has a value the behavior is determined by the `preference` argument to `load`. If `Preference::PreferEnv` is
   given, an environment value will be taken like (2) in all cases. If the environment value is not available it
   will use the YAML file's given key value. If `Preference::PreferYaml` is given, it will take the YAML file always.


## Notes

The parser does not currently support arrays.

The YAML parser is recursive. As a result there is a stack-size limit to the depth of nesting that can be handled.


## Examples

### Load a File with Environment Preference

```rust
use config::Preference;
use config::load;
let configuration = load("path/to/yaml/file.yaml",
                         Some(Preference::PreferEnv))?;
```

### Load a File with YAML Preference

```rust
use config::Preference;
use config::load;
let configuration = load("path/to/yaml/file.yaml",
                         Some(Preference::PreferYaml))?;
```

or

```rust
use config::load;
let configuration = load("path/to/yaml/file.yaml", None)?;
```

### Accessing Values

Values are stored in an enum representing the type.

```rust
use config::load;
let configuration = load("path/to/yaml/file.yaml", None)?;
let val = *config["TEST_ENVIRONMENT_VARIABLE"].as_i64().unwrap();
```

Due to Rust's strict typing you will be responsible for knowing ahead of time what can populate your variables.
Even though `unwrap` is shown here it is _highly_ recommend you use `match` to compensate for this.


