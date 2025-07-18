# workspace-unused-deps

Finds unused code within a workspace. To do this it parses the json output from `cargo-doc` and then performs a plaintext search on all Rust files in the workspace to see if items are actually used.

This is a quick and dirty projects with many false positives and incomplete functionality. Contributions welcome!

## Usage

```bash
$ git clone https://github.com/Nutomic/workspace-unused.git
$ cd workspace-unused
$ cargo run -- -w ../lemmy/lemmy -f full
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
Running `target/debug/workspace-unused-deps -w ../lemmy/lemmy -f full`

Looking for unused code in crates/utils
===


Struct MentionData in crates/utils/src/utils/mention.rs is unused
Struct LemmyErrorTypeIter in crates/utils/src/error.rs is unused
Struct EmailConfig in crates/utils/src/settings/structs.rs is unused
Struct DatabaseConfig in crates/utils/src/settings/structs.rs is unused
Struct FederationErrorIter in crates/utils/src/error.rs is unused
Struct SetupConfig in crates/utils/src/settings/structs.rs is unused

Looking for unused code in crates/db_schema
===

...
```

## License

[AGPL](./LICENSE) 