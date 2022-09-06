# Dynamic Configurator for Rust

The dynamic configuration library is simple: it takes an arbitrary YAML file, parses it into a
common format, and returns a dictionary of configurations and their keys. It supports environment
loading as well.

Take for example the following:

```yaml
database:
  name: null
  username: null
  password: null
  port: null
logging:
  level: 'INFO'
performance:
  threads: 8
```

`config` will parse this into a dictionary as follows:

1. Each key in the dictionary will be given a name `TOP_LEVEL_SUB_LEVEL(S)_KEY_NAME`.
2. For keys with `null` it will look in the environment with the name given by the dictionary. For
   example, database name above will be looked for in `DATABASE_NAME`.
   If it cannot find it in the environment it will panic.
3. For keys with values, the value is taken. If a key in the environment exists with the same
   key it will ignore it. To prefer the environment, pass `Settings::PREFER_ENVIRONMENT` to the
   last argument of `Config::load`.

  **NOTE:** This only supports single YAML documents. It will not load any more than the first
  document in a multi-document yaml.