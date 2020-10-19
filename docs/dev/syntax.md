# Untyped syntax nodes

Traditionally, linters operate directly on an AST. This works fine for most cases if you have a visitor, however, it is too constrained for a powerful linter. ASTs have very significant downsides which include: 

- Often expensive to clone and pass around 
- Node traversal is impossible without runtime typing, and even with runtime typing it is extremely constrained. 
- No straightforward way to obtain individual tokens or child nodes
- Lossy, no way to cleanly and efficiently represent whitespace inside of them, which is crucial for stylistic linting.
- Require explicit handling of their structures in the parser
- Mutable, which is fine for most cases but you really do not want mutable trees for static analysis in a language without a GC. 
- No way to go back to source without a formatter or expensive storage of original source code
- Makes error recovery for a parser very constrained 

To counteract this, RSLint uses a significantly different representation for ASTs, which are called green trees/syntax nodes. At the core, a tree
is immutable and is represented by an immutable "green tree", a tree consists of green nodes which contain children, which can be green nodes and green tokens. [Syntax nodes](https://docs.rs/rowan/0.10.0/rowan/api/struct.SyntaxNode.html) are wrappers consisting of a `Rc` housing some node data in the green tree. [Syntax tokens](https://docs.rs/rowan/0.10.0/rowan/api/struct.SyntaxToken.html) are the same thing but for tokens in the green tree. This means our AST and nodes are all immutable, it also means we can cheaply clone nodes and tokens and pass them around to individual analyzers (e.g. scope analysis) and functions without worrying about memory usage. Nodes and tokens are untyped, which means traversal of the syntax tree from a token is dead simple as you can see from the documentation on syntax node. This gets rid of the issue of complex node traversal being nearly impossible in typed ASTs.

Well does this really make a difference you might be asking. It sure does! for example, the deno_lint (a traditional AST based linter) logic for [no-await-in-loop](../rules/errors/no-await-in-loop.md) is [235 lines](https://github.com/denoland/deno_lint/blob/master/src/rules/no_await_in_loop.rs), while the rslint logic is [35 lines](https://github.com/RDambrosio016/RSLint/blob/master/crates/rslint_core/src/groups/errors/no_await_in_loop.rs). This is because deno lint cannot get the ancestors of an ast node so it has to take a recursive top-down approach, handling every possible async function case. Instead, with untyped nodes we can freely get the ancestors and check them recursively.

Obviously, linting also requires typed structures, in this case, typed AST nodes are simple wrappers on top of a single syntax node. AST nodes simply define functions which traverse the node and get specific productions, which are then casted to another node. The conversion to and from untyped to typed is zero cost, which is great for a parallelized linter which needs to share AST nodes. Another thing about AST nodes is each function for properties returns an optional value (or iterator), because unlike other linters, rslint can fully recover from pretty much all parser errors and still lint the resulting tree. 

Moreover, untyped nodes allow us to grab specific parts of a node, which is particularly important for rslint because it tries to emit the best diagnostics possible, which involves labeling specific parts.

And finally, untyped nodes can be lowered down to both lossy and lossless tokens, which allow us to do many cool things not doable without storing all tokens, which is expensive without interning (green trees do this automatically). These include:

- Lexical equality, which is better than string equality, because according to string equality, `foo .bar` and `foo.bar` are different 
- Lexical syntax highlighting (you can see this in action using `rslint explain some-rule-here`) 
- Lexical checking, such as being able to check the function name with `&["Object", ".", "defineProperty"]` 
- Easy whitespace checking
- Easy comment checking
