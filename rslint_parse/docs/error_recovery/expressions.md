The RSLint Parser will try to recover from certain erroneous syntactical productions. Many of the recoveries result in opinionated results.
Each error recovery tries to be sensible about the result of the recovery.

# Primary Expressions

## Lexer is finished

If the Lexer is finished, the primary expression subparser will emit an error stating an expression was expected but EOF was found.

## Invalid token

An InvalidToken from the lexer is parsed as an identifier token

A production such as `6 + /aaa/gf` is parsed as:

```
BinaryExpr
├── Left
│   └── Number
└── Right
    └── Identifier
```

Due to the fact that `/aaa/gf` is an invalid token because of the invalid flag, not a RegEx.

# Suffixes

## No identifier name after dot

Productions such as `foo.` are invalid, since an identifier name is expected after the dot.
The Parser recovers from this by ignoring the expression and simply returning `foo`.

# Unary Expressions

## Invalid Assign Target

Productions such as `true++` or `--7` are invalid, since `true` and `7` are not valid targets for the operation.
The Parser recovers from this by still parsing the expression despite the target being invalid.

## Linebreak between target and postfix update

According to the ECMAScript specification, a linebreak between a LHS expression and a postfix update expression is invalid.
The Parser throws no errors for this due to the fact that this is valid and not an error:

```js
let foo = ++bar
++foo
```

# Conditional Expressions

## Missing colon

Conditional expressions require a colon after the `if_true` expression, therefore `foo ? bar baz` is invalid.
The parser recovers from this by assuming the colon was right after the leading whitespace.

E.G:

```js
let a = foo ?
     bar 
     baz;
//  ^ Colon assumed to be here.
// the linebreak and indentation was consumed by the leading whitespace
```

# Assignment Expressions

## Invalid Assign Target

Assignment expressions take a left hand side expression for the target, therefore, `true += 5` is an invalid production.
For the purposes of error recovery, the parser still correctly parses this as an assignment expression.
The checking for an assignment token is done by peeking, therefore there are no side effects to trying to parse as assignment.

# Comma Expressions

A Production such as `let a = foo,` is invalid if the production after the comma 