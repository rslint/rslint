//! File watching and incremental relinting facilities.

use crate::*;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use rslint_core::{incremental::incrementally_relint, LintResult};
use std::sync::mpsc::channel;
use std::time::Duration;

pub fn watch<W: Watcher>(walker: &FileWalker, watcher: &mut W) {
    for file in walker
        .files
        .iter()
        .filter_map(|(_, x)| Some(x).filter(|file| file.path.is_some()))
    {
        let _ = watcher.watch(file.path.to_owned().unwrap(), RecursiveMode::NonRecursive);
    }
}

#[allow(unused_must_use)]
pub fn start_watcher(
    mut walker: FileWalker,
    mut results: Vec<(LintResult, &JsFile)>,
    config: Option<&crate::config::Config>,
) {
    let (tx, rx) = channel();

    let mut watcher = if let Ok(w) = watcher(tx, Duration::from_millis(100)) {
        w
    } else {
        return lint_err!("failed to create file watcher, exiting");
    };
    watch(&walker, &mut watcher);

    loop {
        let event = rx.recv();
        if event.is_err() {
            continue;
        };
        match event.unwrap() {
            DebouncedEvent::Remove(path) | DebouncedEvent::Rename(path, _) => {
                results.retain(|(_, file)| file.path != Some(path.clone()));
                watcher.unwatch(path);
            }
            // TODO: This wont work for rules which rely on the context of multiple files
            // and we will have to relint all of the files with all non-CstRule rules
            DebouncedEvent::Write(path) => {
                let (old_res, file) = results.iter().find(|(_, file)| {
                    file.path
                        .as_ref()
                        .map_or(false, |x| x.file_name() == path.file_name())
                }).expect("Tried to get previous result in watcher, but event triggered on an file not included in the linting session");
                walker.maybe_update_file_src(path.clone());

                let file_id = file.id;
                let res = incrementally_relint(old_res.clone(), &walker.files[&file.id].source);
                if let Err(diag) = res {
                    emit_diagnostic(diag, &walker);
                    continue;
                };
                let new = res.unwrap();
                let results = results
                    .clone()
                    .into_iter()
                    .map(|(res, _)| res)
                    .map(|x| {
                        if x.file_id == file_id {
                            new.to_owned()
                        } else {
                            x
                        }
                    })
                    .collect();

                print_results(results, &walker, config);
                println!("{}", "watching for changes...\n".white());
            }
            _ => {}
        }
    }
}
