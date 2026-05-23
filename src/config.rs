use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Clone, Copy)]
pub enum Comment {
    Single(&'static str),
    Double(DoubleSE),
}

#[derive(Clone, Copy)]
pub struct DoubleSE {
    pub open: &'static str,
    pub close: &'static str,
}

pub struct Config {
    pub name: &'static str,
    pub ext: &'static str,
    pub comment_literal: &'static [Comment],
}

const POPULAR_COMMENT_LITERAL: &[Comment] = &[
    Comment::Single("//"),
    Comment::Double(DoubleSE {
        open: "/*",
        close: "*/",
    }),
];

static LANG_BY_EXT: OnceLock<HashMap<&'static str, &'static Config>> = OnceLock::new();

pub fn lang_by_ext() -> &'static HashMap<&'static str, &'static Config> {
    LANG_BY_EXT.get_or_init(|| {
        let mut map = HashMap::new();

        for config in PROGRAMMING_LANGUAGES_CONFIG.iter() {
            map.insert(config.ext, config);
        }

        map
    })
}

const PROGRAMMING_LANGUAGES_CONFIG: &[Config] = &[
    Config {
        name: "Rust",
        ext: "rs",
        comment_literal: POPULAR_COMMENT_LITERAL,
    },
    Config {
        name: "Go",
        ext: "go",
        comment_literal: POPULAR_COMMENT_LITERAL,
    },
    Config {
        name: "C++",
        ext: "cpp",
        comment_literal: POPULAR_COMMENT_LITERAL,
    },
    Config {
        name: "C",
        ext: "c",
        comment_literal: POPULAR_COMMENT_LITERAL,
    },
    Config {
        name: "Javascript",
        ext: "js",
        comment_literal: POPULAR_COMMENT_LITERAL,
    },
    Config {
        name: "Java",
        ext: "java",
        comment_literal: POPULAR_COMMENT_LITERAL,
    },
    Config {
        name: "Kotlin",
        ext: "kt",
        comment_literal: POPULAR_COMMENT_LITERAL,
    },
    Config {
        name: "Typescript",
        ext: "ts",
        comment_literal: POPULAR_COMMENT_LITERAL,
    },
    Config {
        name: "Python",
        ext: "py",
        comment_literal: &[
            Comment::Single("#"),
            Comment::Double(DoubleSE {
                open: r#"""""#,
                close: r#"""""#,
            }),
        ],
    },
    Config {
        name: "Shell",
        ext: "sh",
        comment_literal: &[Comment::Single("#")],
    },
    Config {
        name: "Ruby",
        ext: "rb",
        comment_literal: &[
            Comment::Single("#"),
            Comment::Double(DoubleSE {
                open: "=begin",
                close: "=end",
            }),
        ],
    },
    Config {
        name: "Markdown",
        ext: "md",
        comment_literal: &[Comment::Double(DoubleSE {
            open: "<!--",
            close: "-->",
        })],
    },
    Config {
        name: "HTML",
        ext: "html",
        comment_literal: &[Comment::Double(DoubleSE {
            open: "<!--",
            close: "-->",
        })],
    },
];
