use regex::Regex;
use serde::Deserialize;
use std::fs::read_to_string;
use std::io;
use std::path::PathBuf;
use walkdir::WalkDir;

const BASE_PATH: &str = "xtask/src/coverage/test262/test";

#[derive(Debug, Clone)]
pub struct TestFile {
    pub meta: MetaData,
    pub code: String,
    pub path: PathBuf,
}

/// Representation of the YAML metadata in Test262 tests.
// taken from the boa project
#[derive(Debug, Clone, Deserialize)]
pub struct MetaData {
    pub description: Box<str>,
    pub esid: Option<Box<str>>,
    pub es5id: Option<Box<str>>,
    pub es6id: Option<Box<str>>,
    #[serde(default)]
    pub info: Box<str>,
    #[serde(default)]
    pub features: Box<[Box<str>]>,
    #[serde(default)]
    pub includes: Box<[Box<str>]>,
    #[serde(default)]
    pub flags: Box<[TestFlag]>,
    #[serde(default)]
    pub negative: Option<Negative>,
    #[serde(default)]
    pub locale: Box<[Box<str>]>,
}

/// Negative test information structure.
#[derive(Debug, Clone, Deserialize)]
pub struct Negative {
    pub phase: Phase,
    #[serde(rename = "type")]
    pub error_type: Box<str>,
}

/// Individual test flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TestFlag {
    OnlyStrict,
    NoStrict,
    Module,
    Raw,
    Async,
    Generated,
    #[serde(rename = "CanBlockIsFalse")]
    CanBlockIsFalse,
    #[serde(rename = "CanBlockIsTrue")]
    CanBlockIsTrue,
    #[serde(rename = "non-deterministic")]
    NonDeterministic,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Parse,
    Early,
    Resolution,
    Runtime,
}

fn read_metadata(code: &str) -> io::Result<MetaData> {
    use once_cell::sync::Lazy;

    /// Regular expression to retrieve the metadata of a test.
    static META_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"/\*\-{3}((?:.|\n)*)\-{3}\*/"#)
            .expect("could not compile metadata regular expression")
    });

    let yaml = META_REGEX
        .captures(code)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "no metadata found"))?
        .get(1)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "no metadata found"))?
        .as_str();

    serde_yaml::from_str(yaml).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn get_test_files() -> impl Iterator<Item = TestFile> {
    WalkDir::new(BASE_PATH)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let code = read_to_string(entry.path()).ok()?;
            let meta = read_metadata(&code).ok()?;
            let path = entry.into_path();
            Some(TestFile { code, meta, path }).filter(|file| file.meta.features.is_empty())
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestResult {
    pub fail: Option<FailReason>,
    pub path: PathBuf,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FailReason {
    IncorrectlyPassed,
    IncorrectlyErrored,
    InfiniteRecursion,
}
