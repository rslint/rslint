RSLint-parse is an extremely fast error tolerant, lossless, JavaScript parser written in Rust.
No whitespace is ignored in the parsing of the syntax tree, and it is represented completely in the final CST (Concrete Syntax Tree).
This differs from a traditional AST which is a lossy representation of the program.

# Whitespace

## Lexing

The RSLint-parse Lexer reads characters which are defined by the ECMAScript specification to be classified as whitespace into their own tokens.
Multiple whitespace in a row are read into a single token, however, linebreak tokens contain only a single linebreak.
Comments are also read into tokens as `MultilineComment` and `InlineComment` tokens.

## Parsing

Every single node contains information about the leading and trailing whitespace of each token in the node.
RSLint-parse follows the rules of swift's trivia system, which you can read about [here](https://github.com/apple/swift/tree/master/lib/Syntax#trivia).

Tokens have leading and trailing whitespace, leading whitespace consumes all linebreaks, whitespace, and comments up to the token.
For example, the leading whitespace for `foo`:

```
  

  foo
```

is 2 spaces, 2 linebreaks, then 2 more spaces.
Trailing whitespace on the other hand, consumes all whitespace, and comments up to (but not including) the first linebreak.
for example, the trailing whitespace for `foo`:

```
foo  
bar
```

is only 2 spaces. and the leading whitespace for `bar` is a single linebreak.

### Practical example

Lets say we have an assignment expression such as `foo+= bar`, and your program would like to enforce a single space between `foo` and `+=`.
The parser would emit a tree which looks like:

```
Assign(
    AssignmentExpr {
        span: "foo+= bar",
        left: Identifier(
            LiteralExpr {
                span: "foo",
                whitespace: LiteralWhitespace {
                    before: "",
                    after: "",
                },
            },
        ),
        right: Identifier(
            LiteralExpr {
                span: "bar",
                whitespace: LiteralWhitespace {
                    before: "",
                    after: "",
                },
            },
        ),
        op: AssignOp(
            AddAssign,
        ),
        whitespace: OperatorWhitespace {
            before: "",
            after: " ",
        },
    },
)```

**Keep in mind the span properties are Span structs in reality, not strings, it was edited for the purpose of the example**

once we have the Assign expression we can check the trailing whitespace of `foo` using `expr.left.0.whitespace.after`, the whitespace in reality is a `Span`
instance, therefore we are able to check 