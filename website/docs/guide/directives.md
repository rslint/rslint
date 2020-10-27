# Directives 

Directives are comments which are used to control the linter's behavior from inside source code files. Directives may contain multiple commands,
and may contain any comments after `--` to specify why the directive is needed. Directives placed before any statement or declaration apply to the entire file, and those placed on top of statements or declarations apply only to that node. 

Each directive should start with `rslint-` followed by a command, you can include multiple commands by separating them with `-`. 

## Ignore commands 

`ignore` commands allow you to ignore rules for the entire file, ignore rules for a node, ignore all rules for a node, or ignore an entire file. Ignore commands are simply `ignore` followed by a comma separated list of rule names.

### Examples

Ignoring the entire file:

```js
// rslint-ignore

if (true) {}
```

Ignoring a rule for the entire file:

```js
// rslint-ignore no-empty

if (foo) {}
```

Ignoring a rule for a specific statement or declaration:

```js
// rslint-ignore no-empty
if (foo) {}
```

Ignoring all rules for a statement or declaration:

```js
// rslint-ignore
if (true) {}
```
