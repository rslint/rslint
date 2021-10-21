# Config

RSLint is fully configurable, you can configure the linter through a `rslintrc.toml` or `rslintrc.json` file.

## Rules

You can configure what rules the linter runs using the `rules` field.
The `rules` field can take 4 keys, these are:

- `allowed`: an array of strings of rules which are explicitly allowed and will not be run.
- `errors`: an object where each key is a rule name, and the value is the rule's configuration options (or `{}` if no config). These rules will be treated as errors.
- `warnings`: same as `errors` but the rules will be treated as warnings.
- `groups`: an array of strings where each string is the name of a [rule group](../rules). All of the rules of each group will be treated as errors.

Rule names can be in any case, e.g. `no-empty`, `noEmpty`, `NoEmpty`, and `no_empty` all work. However it is strongly reccomended to keep a consistent case!

These fields above are listed in terms of precedence.

For instance:

```toml
[rules]
allowed = ["no-empty"]
groups = ["errors"]

[rules.errors]
no-empty = { disallowEmptyFunctions = true }

[rules.warnings]
no-empty = {}
```

```json
{
  "rules": {
    "allowed": ["no-empty"],
    "groups": ["errors"],
    "errors": {
      "no-empty": { "disallowEmptyFunctions": true }
    },
    "warnings": {
      "no-empty": {}
    }
  }
}
```

In this case `no-empty` would not be run, because `allowed` always takes precedence. If `allowed` was not there then the `no-empty` in `rules.errors` would
be run. if that was not there then the configuration in `rules.warnings` would be used. if that was not there then the rule would be run at error level because it is included in `errors`.

The linter will warn you if a rule config is being ignored because of precedence.

### Examples

Enabling all rules in the `errors` group:

```toml
[rules]
groups = ["errors"]
```

```json
{
  "rules": {
    "groups": ["errors"]
  }
}
```

Enabling all rules in the `errors` group but allowing `no-empty`:

```toml
[rules]
groups = ["errors"]
allowed = ["no-empty"]
```

```json
{
  "rules": {
    "groups": ["errors"],
    "allowed": ["no-empty"]
  }
}
```

Enabling all rules in the `errors` group but making `no-empty` a warning without configuration:

```toml
[rules]
groups = ["errors"]

[rules.warnings]
no-empty = {}
```

```json
{
  "rules": {
    "groups": ["errors"],
    "warnings": {
      "no-empty": {}
    }
  }
}
```

Enabling `no-empty` with a configuration and enabling `for-direction` as an error:

```toml
[rules.errors]
for-direction = {}
no-empty = { disallowEmptyFunctions = true }
```

```json
{
  "rules": {
    "errors": {
      "for-direction": {},
      "no-empty": { "disallowEmptyFunctions": true }
    }
  }
}
```

or

```toml
[rules.errors]
for-direction = {}

[rules.errors.no-empty]
disallowEmptyFunctions = true
```

```json
{
  "rules": {
    "groups": ["for-direction"],
    "errors": {
      "no-empty": { "disallowEmptyFunctions": true }
    }
  }
}
```
