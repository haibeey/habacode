# habacode

A source-code line counter. Walks a path, counts non-blank, non-comment lines per file, and reports totals by language.

## How it works

The main pilot is `src/config.rs` — it prepares the path of what is to come: for each supported language it declares the file extension and the comment markers (single-line and block).

Next is the scanner in `src/code.rs`. It uses a constant buffer of 1 MB. For each file we read into that buffer, walk through it ignoring comments based on the predefined config, and tally lines that contain code. The same 1 MB buffer is reused for every file in the walk, so memory stays bounded regardless of how large the tree or any single file is.

We currently have a limited set of programming languages defined in `config.rs`. A quick Google search shows over 700 languages exist. The key design here is that this can be easily extended to add any language while using a single binary to hold all information on the numerous programming languages — adding a new language is a single entry in `PROGRAMMING_LANGUAGES_CONFIG`.

## Install

```
curl -fsSL https://raw.githubusercontent.com/haibeey/habacode/main/install/install.sh | bash
```

Downloads the matching prebuilt binary for your OS/arch from the latest GitHub release and installs it to `/usr/bin` (Linux) or `/usr/local/bin` (macOS).

## Usage

```
habacode <path> [-d|--max-depth N]
```

- `<path>` — file or directory to count.
- `-d`, `--max-depth N` — how deep to recurse (default `50`). `0` means the path itself only.

Example:

```
$ habacode src
Rust  453
```

Files with extensions not in the config are silently skipped. Symlinks are not followed.

## Supported languages

Rust, Go, C, C++, JavaScript, TypeScript, Java, Kotlin, Python, Ruby, Shell, Markdown, HTML.

## Adding a language

Append a `Config` entry to `PROGRAMMING_LANGUAGES_CONFIG` in `src/config.rs`:

```rust
Config {
    name: "Ruby",
    ext: "rb",
    comment_literal: &[
        Comment::Single("#"),
        Comment::Double(DoubleSE { open: "=begin", close: "=end" }),
    ],
},
```

`comment_literal` is a slice, so a language with several comment styles can list them all.

## Build from source

```
cargo build --release
./target/release/habacode <path>
```

Requires Rust (edition 2024). No runtime dependencies.
