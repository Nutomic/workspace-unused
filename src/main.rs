use cargo_toml::Manifest;
use clap::Parser;
use grep::searcher::Sink;
use std::error::Error;
use std::fs::File;
use walkdir::DirEntry;
use workspace_unused_deps::{
    parsed::ItemDocsMerged,
    rustdoc::{ApiDocs, ApiItemInner},
};
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
    workspace_root: String,
    #[arg(short, long)]
    features: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let manifest = Manifest::from_path(format!("{}/Cargo.toml", args.workspace_root))?;
    let members = manifest.workspace.unwrap().members;
    for m in members {
        let res = unused_in_crate(&m, &args.workspace_root, &args.features);
        if let Err(e) = res {
            println!("Error in {} while scanning for unused code: {e}", m);
        }
    }
    Ok(())
}

fn unused_in_crate(
    crate_path: &str,
    workspace_root: &str,
    features: &Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let manifest_path = format!("{}/{}/Cargo.toml", workspace_root, crate_path);
    println!("Looking for unused code in {crate_path}");
    let json_path = rustdoc_json::Builder::default()
        .toolchain("nightly-2025-06-22")
        .manifest_path(manifest_path)
        .features(features)
        .silent(true)
        .build()?;

    let file = File::open(json_path)?;
    let jd = &mut serde_json::Deserializer::from_reader(file);
    let mut docs: ApiDocs = serde_path_to_error::deserialize(jd)?;

    let mut merged = vec![];
    for a in docs.index.into_iter() {
        let mut b = docs.paths.remove(&a.0).unwrap_or_default();
        let a = a.1;
        if a.visibility != "public" {
            continue;
        }
        match a.inner {
            ApiItemInner::Function(_) => b.kind = "function".to_string(),
            ApiItemInner::Struct(_) => b.kind = "struct".to_string(),
            ApiItemInner::Other(_) => continue,
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
            "struct" => format!(r"{}", m.name),
            _ => {
                //println!("unsupported kind {}", m.kind);
                continue;
            }
        };

        let found: Vec<_> = search(pattern, &workspace_root)?
            .into_iter()
            .filter(|f| !f.ends_with(&m.span.filename))
            .collect();

        if found.is_empty() {
            match m.kind.as_str() {
                "function" => println!("Function {}() in {} is unused", m.name, m.span.filename),
                "struct" => println!("Struct {} in {} is unused", m.name, m.span.filename),
                _ => {}
            };
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
    for result in walker.filter_entry(|e| include_entry(e)) {
        let dent = match result {
            Ok(dent) => dent,
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        };
        let is_rs_file = dent
            .file_name()
            .to_str()
            .map(|s| s.ends_with(".rs"))
            .unwrap_or(false);
        if !is_rs_file {
            continue;
        }
        let mut out = MySink::default();
        let result = searcher.search_path(&matcher, dent.path(), &mut out);
        if let Err(err) = result {
            eprintln!("{}: {}", dent.path().display(), err);
        }
        if out.found {
            found_in_paths.push(dent.path().to_string_lossy().to_string());
        }
    }
    Ok(found_in_paths)
}

fn include_entry(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_str();
    let is_hidden = name.map(|s| s.starts_with(".")).unwrap_or(false);
    let is_target = name.map(|s| s == "target").unwrap_or(false);
    !is_hidden && !is_target
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
