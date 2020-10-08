# Docgen

Documentation for the rules folder is not manually written, documentation is written as rust doc comments
in source files (in the lint declaration). And a [docgen script](https://github.com/RDambrosio016/RSLint/blob/master/xtask/src/docgen/mod.rs) is used to generate the user facing docs.
The docgen script allows us to make rustdoc documentation, as well as user facing documentation automatically.

Docgen goes through a few steps to collect docs:

- The script crawls the [groups directory](https://github.com/RDambrosio016/RSLint/tree/master/crates/rslint_core/src/groups), for every group it will:  
   _ Collect the group name by looking at the `group!` invocation in `mod.rs`  
   _ For each rule file it will then:  
   _ Collect the `declare_lint!` invocation, from this it grabs the main documentation, replacing all `ignore` code blocks with `js`.
  it will also collect any public config fields and their corresponding documentation.  
   _ Collect the possible `rule_tests!` invocation, for each err and ok test, if it is not marked with `/// ignore` then it  
   will be used in `More invalid examples` and `More valid examples` sections.  
   _ Append the rule name to the top of the file.  
   _ Append the main documentation.  
   _ Build a table of config fields (if any).  
   _ Append any `More invalid examples` and `More valid examples` sections built from tests.  
   _ Append a hyperlink to the source code  
   _ Generate a README for the group, with a table of all the rules, including the first sentence of each rule's doc as a description.  
   it also links each rule's markdown file as a hyperlink.  
   \* The script collects all groups which have been collected and generates this top level document with a table of the groups
  which you might have seen right above ^^

You can run the docgen with either `cargo docgen` or `cargo xtask codegen`.
