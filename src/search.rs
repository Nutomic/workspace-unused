use grep::searcher::Sink;
use std::error::Error;
use walkdir::DirEntry;
use {
    grep::{
        regex::RegexMatcher,
        searcher::{BinaryDetection, SearcherBuilder},
    },
    walkdir::WalkDir,
};

/// Search if the item name is used by any .rs file in the workspace
pub fn search(pattern: &str, path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let matcher = RegexMatcher::new_line_matcher(pattern)?;
    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .line_number(false)
        .build();

    let mut found_in_paths = vec![];
    let walker = WalkDir::new(path).into_iter();
    for result in walker.filter_entry(|e| include_entry(e)) {
        let entry = match result {
            Ok(dent) => dent,
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        };
        if entry.file_type().is_dir() {
            continue;
        }
        let mut out = MySink::default();
        let result = searcher.search_path(&matcher, entry.path(), &mut out);
        if let Err(err) = result {
            eprintln!("{}: {}", entry.path().display(), err);
        }
        if out.found {
            found_in_paths.push(entry.path().to_string_lossy().to_string());
        }
    }
    Ok(found_in_paths)
}

fn include_entry(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_str();
    let is_hidden = name.map(|s| s.starts_with(".")).unwrap_or(false);
    let is_target = name.map(|s| s == "target").unwrap_or(false);
    let is_dir = entry.file_type().is_dir();
    let is_rs_type = name.map(|s| s.ends_with(".rs")).unwrap_or(false);
    !is_hidden && !is_target && (is_dir || is_rs_type)
}

#[derive(Default)]
struct MySink {
    found: bool,
}

impl Sink for MySink {
    type Error = std::io::Error;

    fn matched(
        &mut self,
        _searcher: &grep::searcher::Searcher,
        _mat: &grep::searcher::SinkMatch<'_>,
    ) -> Result<bool, Self::Error> {
        self.found = true;
        Ok(false)
    }
}
