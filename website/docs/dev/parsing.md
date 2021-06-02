# Parsing

So now that we explained lexing as well as the basic concept behind RSLint's syntax tree, how does the parser even produce that tree? This page answers that question and provides deeper insight as to why the parser is set up how it is.

## Events

The core structure the parser produces is an [`Event`](https://docs.rs/rslint_parser/0.2.1/rslint_parser/enum.Event.html). An event simply describes the events the parser goes through while parsing. These events are then fed to the [`process`](https://docs.rs/rslint_parser/0.2.1/rslint_parser/fn.process.html) function which runs through them and applies them to a `TreeSink`, a structure we will talk about later.

The possible event's values are self explanatory. `Start` tells the tree sink to start a node with a specific kind, it also may include a `forward_parent`, this is used to make nodes which start before other nodes, which is required for exprs such as `5 + 5`. `Finish` tells the sink to finish the current node. `Token` tells the tree sink to append a token to the current node. And finally, `Error` tells the tree sink to record an error or warning that happened during parsing.

Events are cool because they allow us to go through the parsing process without having to explicitly handle the AST structures, parsing functions simply start new nodes, finish nodes, and add tokens. Moreover, events allow us to cheaply backtrack the parser by simply draining the events and resetting the token source cursor back to some place.

## Markers

Another central structure you will come across is [`Marker`](https://docs.rs/rslint_parser/0.2.1/rslint_parser/struct.Marker.html) and its complement, [`CompletedMarker`](https://docs.rs/rslint_parser/0.2.1/rslint_parser/struct.CompletedMarker.html). A marker simply signifies the start of parsing a node in the parser, you start a marker, then you consume tokens, the tokens consumed between creating the marker and completing it now belong to that node. Completed markers are simply structures which represent a parsed node, you can directly turn a completed marker into an AST node using [`parse_marker`](https://docs.rs/rslint_parser/0.2.1/rslint_parser/struct.Parser.html#method.parse_marker), however that is quite expensive so avoid it if you can.

## Tree Sinks

Ok, so we made markers, we consumed the parser to make events, what now? Well the final step to producing a tree is passing it through a [`TreeSink`](https://docs.rs/rslint_parser/0.2.1/rslint_parser/trait.TreeSink.html). A tree sink is an abstraction which can take events and turn them into a tree. [`LosslessTreeSink`](https://docs.rs/rslint_parser/0.2.1/rslint_parser/struct.LosslessTreeSink.html) is the most common Tree Sink, it retains all whitespace by "gluing" or "eating" trivia (whitespace and comments) while making nodes, it also glues some things like comments to nodes (to make directive parsing easier). The other tree sink, [`LossyTreeSink`](https://docs.rs/rslint_parser/0.2.1/rslint_parser/struct.LossyTreeSink.html) consumes events but does not glue whitespace to each node.

## Parses

The final structure in the parsing process is the [`Parse<T>`](https://docs.rs/rslint_parser/0.2.1/rslint_parser/struct.Parse.html), the parse is just a simple structure to manage the result of a parser job. It contains all of the errors produced and the Green tree produced, and can output a typed ast node, or an untyped node.

## Error Recovery

One of the best features of `rslint_parser` is its very powerful error recovery, which allows it to parse any source code and produce an AST no matter how wrong. Since everything in every AST node is optional, we can apply two basic concepts for error recovery:

- If we expect a token, and its not there, just issue and error and go on, we don't need it in the AST, e.g. `if true`
- If we find a token we didn't expect then issue an error and then:
  - If the token is `{` or `}` then we just don't do anything (most of the time) because it can easily be recovered from by parsing
    as a block statement or object literal
  - We define an error recovery set, if the token is in that set, do nothing and return.
  - Otherwise, bump the token and return.

This allows us to have extremely powerful error recovery, but at the same time, we are subject to infinite recursion while trying to recover, and we have run into many bugs which allow that.

Error recovery is a central concept of RSLint, and it allows us to lint incorrect code, unlike other linters. Moreover, error recovery is very important when it comes to language servers, which allows RSLint to lint code on the fly while you are typing, instead of having to wait for syntactically valid code.

## Quick links

- [where the actual JS parsing logic is](https://github.com/rslint/rslint/tree/master/crates/rslint_parser/src/syntax)
- [where the AST logic is](https://github.com/rslint/rslint/tree/master/crates/rslint_parser/src/ast)
- [where the central Parser structure is](https://github.com/rslint/rslint/blob/master/crates/rslint_parser/src/parser.rs)
