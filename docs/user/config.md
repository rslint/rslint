# Linter Configuration

RSLint is fully configurable, you can configure the linter through a `rslintrc.toml` file in the linting directory.

## Syntax

RSLint uses [TOML](https://toml.io/en/) as its standard configuration format. TOML is a simple, minimal, human friendly format similar to INI.
TOML was chosen over JSON for clarity, simplicity, and ease of use.

Here are a few examples of a TOML configuration as opposed to a JSON configuration:

```json
"rules": {
  "allow": ["no-await-in-loop"],
  "groups": ["errors"],
  "errors": {
    "no-empty": { "disallowEmptyFunction": true },
    "for-direction": {}
  },
  "warnings": {
    "getter-return": {}
  }
}
```

TOML equivalent:

```toml
[rules]
allow = ["no-empty"]
groups = ["errors"]

[rules.errors]
no-empty = { disallowEmptyFunction = true }
for-direction = {}

[rules.warnings]
getter-return = {}
```

TOML syntax also allows for:

```toml
[rules]
allow = ["no-empty"]
groups = ["errors"]

[rules.errors]
for-direction = {}

[rules.errors.no-empty]
disallowEmptyFunction = true

[rules.warnings]
getter-return = {}
```

TOML further allows for comments using `# This is a comment` which allows you to explain reasonings behind
configuration fields.

## Rules

You can configure what rules the linter runs using the `rules` field.
The `rules` field can take 4 keys, these are:

- `allow`: an array of strings of rules which are explicitly allowed and will not be run.
- `errors`: an object where each key is a rule name, and the value is the rule's configuration options (or `{}` if no config). These rules will be treated as errors.
- `warnings`: same as `errors` but the rules will be treated as warnings.
- `groups`: an array of strings where each string is the name of a [rule group](./rules). All of the rules of each group will be treated as errors.

Rule names can be in any case, e.g. `no-empty`, `noEmpty`, `NoEmpty`, and `no_empty` all work. However it is strongly reccomended to keep a consistent case!

These fields above are listed in terms of precedence.

For instance:

```toml
[rules]
allow = ["no-empty"]
groups = ["errors"]

[rules.errors]
no-empty = { disallowEmptyFunctions = true }

[rules.warnings]
no-empty = {}
```

In this case `no-empty` would not be run, because `allow` always takes precedence. If allow was not there then the `no-empty` in `rules.errors` would
be run. if that was not there then the configuration in `rules.warnings` would be used. if that was not there then the rule would be run at error level because it is included in `errors`.

The linter will warn you if a rule config is being ignored because of precedence.

### Examples

Enabling all rules in the `errors` group:

```toml
[rules]
groups = ["errors"]
```

Enabling all rules in the `errors` group but allowing `no-empty`:

```toml
[rules]
groups = ["errors"]
allow = ["no-empty"]
```

Enabling all rules in the `errors` group but making `no-empty` a warning without configuration:

```toml
[rules]
groups = ["errors"]

[rules.warnings]
no-empty = {}
```

Enabling `no-empty` with a configuration and enabling `for-direction` as an error:

```toml
[rules.errors]
for-direction = {}
no-empty = { disallowEmptyFunctions = true }
```

or

```toml
[rules.errors]
for-direction = {}

[rules.errors.no-empty]
disallowEmptyFunctions = true
```
