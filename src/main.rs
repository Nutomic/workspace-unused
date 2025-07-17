use clap::Parser;
use grep::searcher::Sink;
use std::error::Error;
use std::fs::File;
use walkdir::DirEntry;
use workspace_unused_deps::{ApiDocs, ItemDocsMerged};
use {
    grep::{
        regex::RegexMatcher,
        searcher::{BinaryDetection, SearcherBuilder},
    },
    walkdir::WalkDir,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    manifest_path: String,

    #[arg(short, long)]
    workspace_root: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let json_path = rustdoc_json::Builder::default()
        .toolchain("nightly-2025-06-22")
        .manifest_path(args.manifest_path)
        .all_features(true)
        .silent(true)
        .build()
        .unwrap();

    dbg!(&json_path);
    let file = File::open(json_path)?;
    let mut docs: ApiDocs = serde_json::from_reader(file)?;

    let mut merged = vec![];
    for a in docs.index.into_iter() {
        let mut b = docs.paths.remove(&a.0).unwrap_or_default();
        let a = a.1;
        if a.inner.function.is_some() {
            b.kind = "function".to_string();
        }
        if a.visibility != "public" {
            continue;
        }
        if let (Some(name), Some(span)) = (a.name, a.span) {
            merged.push(ItemDocsMerged {
                name,
                span,
                visibility: a.visibility,
                kind: b.kind,
            });
        }
    }

    for m in merged {
        let pattern = match m.kind.as_str() {
            "function" => format!(r"{}\(", m.name),
            _ => {
                //println!("unsupported kind {}", m.kind);
                continue;
            }
        };

        let found: Vec<_> = search(pattern, &args.workspace_root)?
            .into_iter()
            .filter(|f| !f.ends_with(&m.span.filename))
            .collect();

        if found.is_empty() {
            println!("Function {}() in {} is unused", m.name, m.span.filename);
        }
    }

    Ok(())
}

fn search(pattern: String, path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let matcher = RegexMatcher::new_line_matcher(&pattern)?;
    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .line_number(false)
        .build();

    let mut found_in_paths = vec![];
    let walker = WalkDir::new(path).into_iter();
    for result in walker.filter_entry(|e| !is_hidden(e) && !is_target(e)) {
        let dent = match result {
            Ok(dent) => dent,
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        };
        if !dent.file_type().is_file() || dent.file_name().to_str().unwrap().starts_with('.') {
            continue;
        }
        let mut out = MySink::default();
        let result = searcher.search_path(&matcher, dent.path(), &mut out);
        if let Err(err) = result {
            eprintln!("{}: {}", dent.path().display(), err);
        }
        if out.found {
            found_in_paths.push(dent.path().to_str().unwrap().to_string());
        }
    }
    Ok(found_in_paths)
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn is_target(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s == "target")
        .unwrap_or(false)
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
