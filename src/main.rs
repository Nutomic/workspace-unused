use cargo_toml::Manifest;
use clap::Parser;
use std::error::Error;
use std::fs::File;
use workspace_unused_deps::{
    rustdoc::{ApiDocs, ItemKind},
    search::search,
    ItemDocsMerged,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Root folder of the workspace to analyze
    #[arg(short, long)]
    workspace_root: String,

    /// List of feature flags to enable for all crates
    #[arg(short, long)]
    features: Vec<String>,

    /// Whether or not to pass `--quiet` to `cargo rustdoc`
    #[arg(short, long, action)]
    silent: bool,
}

/// Iterate workspace members
fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let manifest = Manifest::from_path(format!("{}/Cargo.toml", args.workspace_root))?;
    let members = manifest.workspace.unwrap().members;
    for m in members {
        let res = unused_in_crate(&m, &args);
        if let Err(e) = res {
            println!("Error in {} while scanning for unused code: {e}", m);
        }
    }
    Ok(())
}

/// Generate api docs for the crate, then print any unused items
fn unused_in_crate(crate_path: &str, args: &Args) -> Result<(), Box<dyn Error>> {
    println!("\nLooking for unused code in {crate_path}\n===\n\n");

    // TODO: Instead of generating docs for each workspace member one by one, we
    //       could call `cargo doc --workspace` but rustdoc_json doesnt have that param.
    let manifest_path = format!("{}/{}/Cargo.toml", args.workspace_root, crate_path);
    let json_path = rustdoc_json::Builder::default()
        .toolchain("nightly-2025-06-22")
        .manifest_path(manifest_path)
        .features(&args.features)
        .quiet(true)
        .silent(args.silent)
        .build()?;

    let file = File::open(json_path)?;
    let jd = &mut serde_json::Deserializer::from_reader(file);
    jd.disable_recursion_limit();
    let mut docs: ApiDocs = serde_path_to_error::deserialize(jd)?;

    let mut merged = vec![];
    for a in docs.index.into_iter() {
        let b = docs.paths.remove(&a.0).unwrap_or_default();
        let a = a.1;
        if a.visibility != "public" || !b.kind.is_supported() {
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
        let found = search(&m.name, &args.workspace_root)?;

        // If item is only used in the same file where it is defined, it can be private instead
        let found: Vec<_> = found
            .into_iter()
            .filter(|f| !f.ends_with(&m.span.filename))
            .collect();

        if found.is_empty() {
            match m.kind {
                ItemKind::Function => {
                    println!("Function `{}()` in {} is unused", m.name, m.span.filename)
                }
                _ => {
                    println!("{:?} `{}` in {} is unused", m.kind, m.name, m.span.filename)
                }
            };
        }
    }
    Ok(())
}
