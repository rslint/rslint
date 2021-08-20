use rslint_parser::parse_text;

mod analyzer;
mod ir;
mod util;

#[test]
fn foo() {
    let mut analyzer = analyzer::Analyzer::from_root(
        parse_text(
            "
          var a = 5;
          {
            {
                var a = 6;
            }
              var a = 76;
          }

    ",
            0,
        )
        .syntax(),
    );
    let scope = analyzer.cur_scope.clone();
    analyzer.analyze_cur_scope();
    println!("{:#?}", scope);
}
