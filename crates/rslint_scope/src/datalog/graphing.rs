use crate::Datalog;
use differential_datalog::ddval::DDValConvert;
use rslint_scoping_ddlog::Indexes;
use std::io::{Result, Write};
use types::{
    ast::{FileId, ScopeId},
    ddlog_std::tuple2,
    inputs::{EveryScope, InputScope},
    name_in_scope::{NameInScope, NameOrigin},
};

impl Datalog {
    pub fn graph_scope(_file: FileId, _scope: ScopeId, _output: &mut impl Write) -> Result<()> {
        todo!()
    }
}

fn _graph_file(file: FileId, datalog: &Datalog, output: &mut impl Write) -> Result<()> {
    writeln!(
        output,
        "digraph {{\n\t\
            graph [compound = true];"
    )?;

    let scopes = datalog
        .query(Indexes::inputs_EveryScopeByFile, Some(file.into_ddvalue()))
        .unwrap()
        .into_iter()
        .map(EveryScope::from_ddvalue);

    for scope in scopes {
        writeln!(
            output,
            "\tsubgraph cluster_scope_{scope} {{\n\t\t\
                label = \"scope #{scope}\";\n\t\t\
                scope_{scope}_head [shape = point, style = invis];",
            scope = scope.scope.id,
        )?;

        let names = datalog
            .query(
                Indexes::name_in_scope_Index_VariablesForScope,
                Some(tuple2(file, scope.scope).into_ddvalue()),
            )
            .unwrap()
            .into_iter()
            .map(NameInScope::from_ddvalue);

        for name in names {
            match name.origin {
                NameOrigin::UserDefined { scope: def } if def == scope.scope => {
                    writeln!(
                        output,
                        "\t\t\"{}_{}_{}\" [label = \"\\\"{}\\\"\", color = crimson, fontcolor = crimson];",
                        scope.scope.id, *name.name, name.declared_in, *name.name,
                    )?;
                }

                NameOrigin::UserDefined { .. } => {
                    writeln!(
                        output,
                        "\t\t\"{}_{}_{}\" [label = \"\\\"{}\\\"\", color = cornflowerblue, fontcolor = cornflowerblue];",
                        scope.scope.id,*name.name, name.declared_in, *name.name,
                    )?;
                }

                _ => {}
            }
        }

        writeln!(output, "\t}}")?;
    }

    let children = datalog
        .query(Indexes::inputs_InputScopeByFile, Some(file.into_ddvalue()))
        .unwrap()
        .into_iter()
        .map(InputScope::from_ddvalue);

    for child in children {
        writeln!(
            output,
            "\tscope_{parent}_head -> scope_{child}_head [\n\t\t\
                ltail = cluster_scope_{parent},\n\t\t\
                lhead = cluster_scope_{child}\n\t\
            ];",
            parent = child.parent.id,
            child = child.child.id,
        )?;
    }

    writeln!(output, "}}")
}

// #[test]
// fn graph_test() {
//     use crate::{Config, ScopeAnalyzer};
//     use rslint_parser::parse_module;
//
//     let file = FileId::new(0);
//     let contents = include_str!("../tests/corpus/TypedArrayConstructor.mjs");
//     let syntax = parse_module(contents, file.id as usize);
//
//     let analyzer = ScopeAnalyzer::new().unwrap();
//     analyzer
//         .analyze(file, &syntax.syntax(), Config::default())
//         .unwrap();
//
//     let mut output = std::io::BufWriter::new(std::fs::File::create("graph.dot").unwrap());
//     _graph_file(file, &*analyzer, &mut output).unwrap();
// }
