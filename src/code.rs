use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use crate::config;

const BUF_SIZE: usize = 1024 * 1024;

pub fn walk(root: &Path, max_depth: usize) -> io::Result<HashMap<&'static str, usize>> {
    let mut totals: HashMap<&'static str, usize> = HashMap::new();
    let mut buf = vec![0u8; BUF_SIZE];
    walk_inner(root, max_depth, 0, &mut buf, &mut totals)?;
    Ok(totals)
}

fn walk_inner(
    path: &Path,
    max_depth: usize,
    depth: usize,
    buf: &mut [u8],
    totals: &mut HashMap<&'static str, usize>,
) -> io::Result<()> {
    let meta = fs::symlink_metadata(path)?;
    let file_type = meta.file_type();

    if file_type.is_file() {
        if let Some((lang, n)) = count_file(path, buf)? {
            *totals.entry(lang).or_default() += n;
        }
        return Ok(());
    }

    if file_type.is_dir() {
        if depth >= max_depth {
            return Ok(());
        }
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            walk_inner(&entry.path(), max_depth, depth + 1, buf, totals)?;
        }
    }

    Ok(())
}

fn count_file(path: &Path, buf: &mut [u8]) -> io::Result<Option<(&'static str, usize)>> {
    let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
        return Ok(None);
    };
    let Some(cfg) = config::lang_by_ext().get(ext) else {
        return Ok(None);
    };

    let mut file = File::open(path)?;
    let mut scanner = Scanner::new(cfg);
    let mut filled = 0;

    loop {
        let n = file.read(&mut buf[filled..])?;
        filled += n;
        let eof = n == 0;

        let consumed = scanner.feed(&buf[..filled], eof);

        if eof {
            break;
        }

        if consumed == 0 && filled == buf.len() {
            return Err(io::Error::other(
                "scan buffer too small for the language's comment markers",
            ));
        }

        buf.copy_within(consumed..filled, 0);
        filled -= consumed;
    }

    Ok(Some((cfg.name, scanner.finish())))
}

struct Scanner<'a> {
    singles: Vec<&'a [u8]>,
    blocks: Vec<(&'a [u8], &'a [u8])>,
    max_marker_len: usize,
    in_block: Option<&'a [u8]>,
    in_line_comment: bool,
    line_has_code: bool,
    count: usize,
}

impl<'a> Scanner<'a> {
    fn new(cfg: &'a config::Config) -> Self {
        let mut singles: Vec<&[u8]> = Vec::new();
        let mut blocks: Vec<(&[u8], &[u8])> = Vec::new();
        for c in cfg.comment_literal {
            match c {
                config::Comment::Single(s) => singles.push(s.as_bytes()),
                config::Comment::Double(d) => blocks.push((d.open.as_bytes(), d.close.as_bytes())),
            }
        }

        singles.sort_by_key(|m| std::cmp::Reverse(m.len()));
        blocks.sort_by_key(|(o, _)| std::cmp::Reverse(o.len()));

        let max_marker_len = singles
            .iter()
            .map(|s| s.len())
            .chain(blocks.iter().flat_map(|(o, c)| [o.len(), c.len()]))
            .max()
            .unwrap_or(0);

        Scanner {
            singles,
            blocks,
            max_marker_len,
            in_block: None,
            in_line_comment: false,
            line_has_code: false,
            count: 0,
        }
    }

    fn feed(&mut self, buf: &[u8], eof: bool) -> usize {
        let reserve = self.max_marker_len.saturating_sub(1);
        let safe_end = if eof {
            buf.len()
        } else {
            buf.len().saturating_sub(reserve)
        };

        let mut i = 0;
        while i < buf.len() {
            if i >= safe_end && !self.in_line_comment {
                break;
            }

            let b = buf[i];

            if b == b'\n' {
                if self.line_has_code {
                    self.count += 1;
                }
                self.in_line_comment = false;
                self.line_has_code = false;
                i += 1;
                continue;
            }

            if self.in_line_comment {
                i += 1;
                continue;
            }

            if let Some(close) = self.in_block {
                if buf[i..].starts_with(close) {
                    self.in_block = None;
                    i += close.len();
                } else {
                    i += 1;
                }
                continue;
            }

            if let Some(marker) = self.singles.iter().find(|m| buf[i..].starts_with(m)) {
                self.in_line_comment = true;
                i += marker.len();
                continue;
            }

            if let Some(&(open, close)) = self.blocks.iter().find(|(o, _)| buf[i..].starts_with(o))
            {
                self.in_block = Some(close);
                i += open.len();
                continue;
            }

            if !b.is_ascii_whitespace() {
                self.line_has_code = true;
            }
            i += 1;
        }

        if eof && self.line_has_code {
            self.count += 1;
            self.line_has_code = false;
        }

        i
    }

    fn finish(self) -> usize {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::lang_by_ext;

    fn count(ext: &str, src: &str) -> usize {
        let cfg = lang_by_ext()
            .get(ext)
            .expect("test ext missing from registry");
        let mut s = Scanner::new(cfg);
        s.feed(src.as_bytes(), true);
        s.finish()
    }

    fn count_chunked(ext: &str, src: &str, chunk_size: usize) -> usize {
        let cfg = lang_by_ext()
            .get(ext)
            .expect("test ext missing from registry");
        let mut s = Scanner::new(cfg);
        let bytes = src.as_bytes();
        let mut buf: Vec<u8> = Vec::new();
        let mut pos = 0;
        loop {
            let want = chunk_size.saturating_sub(buf.len()).min(bytes.len() - pos);
            buf.extend_from_slice(&bytes[pos..pos + want]);
            pos += want;
            let eof = pos == bytes.len();
            let consumed = s.feed(&buf, eof);
            if eof {
                break;
            }
            assert!(
                consumed > 0 || buf.len() < chunk_size,
                "scanner made no progress with a full buffer"
            );
            buf.drain(..consumed);
        }
        s.finish()
    }

    #[test]
    fn counts_basic_lines() {
        assert_eq!(count("rs", "fn main() {}\nlet x = 1;\n"), 2);
    }

    #[test]
    fn skips_blank_lines() {
        assert_eq!(count("rs", "a\n\n\nb\n"), 2);
    }

    #[test]
    fn skips_full_line_comment() {
        assert_eq!(count("rs", "// nope\nlet x = 1;\n"), 1);
    }

    #[test]
    fn counts_line_with_trailing_comment() {
        assert_eq!(count("rs", "let x = 1; // tail\n"), 1);
    }

    #[test]
    fn skips_block_comment_body() {
        assert_eq!(count("rs", "/* a\nb\nc */\nlet x = 1;\n"), 1);
    }

    #[test]
    fn counts_code_around_block_comment() {
        assert_eq!(count("rs", "let x = 1; /* m */ let y = 2;\n"), 1);
    }

    #[test]
    fn counts_final_line_without_newline() {
        assert_eq!(count("rs", "let x = 1;"), 1);
    }

    #[test]
    fn python_hash_and_triple_quote() {
        let src = "x = 1\n# c\n\"\"\"\ndoc\n\"\"\"\ny = 2\n";
        assert_eq!(count("py", src), 2);
    }

    #[test]
    fn html_block_only() {
        let src = "<p>hi</p>\n<!-- c -->\n<p>bye</p>\n";
        assert_eq!(count("html", src), 2);
    }

    #[test]
    fn chunked_matches_whole() {
        let cases: &[(&str, &str)] = &[
            (
                "rs",
                "fn main() {}\n/* a\nb */ let x = 1; // tail\nlet y = 2;\n",
            ),
            (
                "rs",
                "let a = 1;\nlet b = 2; // trailing\n/* split */ let c = 3;",
            ),
            ("py", "x = 1\n# c\n\"\"\"\ndoc\n\"\"\"\ny = 2\n"),
            ("html", "<p>hi</p>\n<!-- block\nspans -->\n<p>bye</p>"),
        ];
        for (ext, src) in cases {
            let whole = count(ext, src);
            // The buffer must hold at least max_marker_len bytes to make progress; our longest
            // marker is "<!--" (4 bytes), so 4 is the smallest viable chunk size.
            for chunk_size in [4, 5, 8, 16, 64] {
                assert_eq!(
                    count_chunked(ext, src, chunk_size),
                    whole,
                    "ext={ext} chunk_size={chunk_size}",
                );
            }
        }
    }

    #[test]
    fn walks_dir_aggregating_by_language() {
        let dir = std::env::temp_dir().join(format!("habacode-walk-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("nested")).unwrap();
        fs::write(dir.join("a.rs"), "fn x() {}\n").unwrap();
        fs::write(dir.join("b.rs"), "// only comment\nlet y = 1;\n").unwrap();
        fs::write(dir.join("nested/c.py"), "x = 1\n").unwrap();
        fs::write(dir.join("README.md"), "hello\n").unwrap();
        fs::write(dir.join("unknown.xyz"), "ignored\n").unwrap();

        let totals = walk(&dir, 50).unwrap();
        assert_eq!(totals.get("Rust").copied(), Some(2));
        assert_eq!(totals.get("Python").copied(), Some(1));
        assert_eq!(totals.get("Markdown").copied(), Some(1));
        assert!(!totals.contains_key("HTML"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn walk_respects_max_depth() {
        let dir = std::env::temp_dir().join(format!("habacode-depth-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("a/b/c")).unwrap();
        fs::write(dir.join("top.rs"), "let a = 1;\n").unwrap();
        fs::write(dir.join("a/b/c/deep.rs"), "let b = 1;\n").unwrap();

        let t1 = walk(&dir, 1).unwrap();
        assert_eq!(t1.get("Rust").copied(), Some(1));

        let t10 = walk(&dir, 10).unwrap();
        assert_eq!(t10.get("Rust").copied(), Some(2));

        fs::remove_dir_all(&dir).unwrap();
    }
}
