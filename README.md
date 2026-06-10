# habacode

A source-code line counter. Walks a path, counts non-blank, non-comment lines per file, and reports totals by language.

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
