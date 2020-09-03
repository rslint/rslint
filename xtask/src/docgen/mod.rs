mod extract;

use crate::project_root;
use extract::*;
use proc_macro2::Span;
use quote::ToTokens;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{read_dir, read_to_string, write};

const GROUPS_ROOT: &str = "rslint_core/src/groups";

pub fn run() {
    for file in read_dir(project_root().join(GROUPS_ROOT))
        .expect("Unreadable groups dir")
        .filter_map(Result::ok)
    {
        if file.file_type().expect("No file type").is_dir() {
            let res = parse_group_mod(
                &read_to_string(file.path().join("mod.rs")).expect("No mod file for group"),
            );
            let meta = res.expect("No group! declaration in group");

            let dir = project_root().join("docs").join(&meta.name);

            let res = extract_group(&meta.name).expect("Failed to extract group rule data");
            let data = group_markdown(&res, &meta);

            write(dir.join("README.md"), data).expect("Failed to write group md");
            for (name, rule) in res {
                write(
                    dir.join(name).with_extension("md"),
                    rule_markdown(rule, &meta),
                )
                .expect("Failed to write rule markdown");
            }
        }
    }
}

pub fn group_markdown(data: &HashMap<String, RuleFile>, group: &Group) -> String {
    let mut ret = String::new();
    ret.insert_str(
        0,
        "<!--\n generated docs file, do not edit by hand, see xtask/docgen \n-->\n",
    );
    ret.push_str(&format!("\n# {}\n\n", group.name));
    ret.push_str(&group.docstring);

    ret.push_str("\n## Rules\n");
    ret.push_str("| Name | Description |\n");
    ret.push_str("| ---- | ----------- |\n");

    for (name, rule) in data {
        let source_file = rule_src(&group.name, &name.replace("-", "_"));
        ret.push_str(&format!(
            "| [`{}`]({}) | {} |\n",
            name,
            source_file,
            first_sentence(&rule.lint_declaration.docstring.clone().unwrap_or_default())
                .unwrap_or_default()
        ));
    }
    ret
}

pub fn rule_src(group_name: &str, rule_name: &str) -> String {
    format!("{}/{}/{}", GROUPS_ROOT, group_name, rule_name)
}

pub fn first_sentence(string: &str) -> Option<&str> {
    string.split("\n").next().map(|x| x.trim())
}

pub fn extract_group(group_name: &str) -> Result<HashMap<String, RuleFile>, Box<dyn Error>> {
    let dir = read_dir(project_root().join(GROUPS_ROOT).join(group_name))?;
    let mut res = HashMap::new();
    for file in dir.filter_map(Result::ok) {
        if let Some(parsed) = parse_rule_file(&read_to_string(file.path())?)? {
            res.insert(file.file_name().to_string_lossy().to_string(), parsed);
        }
    }
    Ok(res)
}

pub fn rule_markdown(rule: RuleFile, group: &Group) -> String {
    let mut ret = rule
        .lint_declaration
        .docstring
        .unwrap_or_default()
        .clone()
        .replace("```ignore", "```js");
    ret.insert_str(
        0,
        "<!--\n generated docs file, do not edit by hand, see xtask/docgen \n-->\n",
    );

    if !rule.lint_declaration.config_fields.is_empty() {
        ret.push_str("\n## Config\n");
        ret.push_str("| Name | Type | Description |\n");
        ret.push_str("| ---- | ---- | ----------- |\n");

        for config in rule.lint_declaration.config_fields.iter() {
            ret.push_str(&format!(
                "| `{}` | {} | {} |\n",
                config.field.ident.as_ref().unwrap(),
                config.field.ty.to_token_stream().to_string(),
                config.docstring.clone().unwrap_or_default()
            ));
        }
    }

    if let Some(tests) = rule.tests {
        if !tests.err_examples.is_empty() {
            ret.push_str("\n<details>\n <summary> More incorrect examples </summary>\n");
            for example in tests.err_examples {
                ret.push_str(&format!(
                    "{}\n```js\n{}\n```\n",
                    example.docstring.unwrap_or_default(),
                    example.source
                ));
            }
            ret.push_str("</details>");
        }
        if !tests.ok_examples.is_empty() {
            ret.push_str("\n<details>\n <summary> More correct examples </summary>\n");
            for example in tests.ok_examples {
                ret.push_str(&format!(
                    "{}\n```js\n{}\n```\n",
                    example.docstring.unwrap_or_default(),
                    example.source
                ));
            }
            ret.push_str("</details>");
        }
    }

    ret.push_str(&format!(
        "\n\n[`Source`]({})",
        rule_src(&group.name, &rule.lint_declaration.name)
    ));
    ret
}
