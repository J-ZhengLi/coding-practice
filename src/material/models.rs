use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// Information about a GitHub repository returned by search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub full_name: String,
    pub html_url: String,
    pub stargazers_count: u32,
    pub description: Option<String>,
    pub language: String,
}

/// File content fetched from GitHub or a tutorial site.
#[derive(Debug, Clone)]
pub struct FileContent {
    pub name: String,
    pub path: String,
    pub content: String,
    pub language: String,
}

/// A curated tutorial site for source code extraction.
#[derive(Debug, Clone)]
pub struct TutorialSource {
    pub name: String,
    pub base_url: String,
    pub language: String,
    pub code_selector: String,
}

/// Curated tutorial sites for each supported language.
pub static CURATED_SITES: LazyLock<Vec<TutorialSource>> = LazyLock::new(|| {
    vec![
        TutorialSource {
            name: "Rust by Example".to_string(),
            base_url: "https://doc.rust-lang.org/rust-by-example/".to_string(),
            language: "rust".to_string(),
            code_selector: "pre > code".to_string(),
        },
        TutorialSource {
            name: "Go by Example".to_string(),
            base_url: "https://gobyexample.com/".to_string(),
            language: "go".to_string(),
            code_selector: ".highlight pre code".to_string(),
        },
        TutorialSource {
            name: "Python Tutorial".to_string(),
            base_url: "https://docs.python.org/3/tutorial/".to_string(),
            language: "python".to_string(),
            code_selector: ".highlight pre".to_string(),
        },
        TutorialSource {
            name: "cppreference".to_string(),
            base_url: "https://en.cppreference.com/w/cpp/language".to_string(),
            language: "cpp".to_string(),
            code_selector: ".mw-content-ltr pre".to_string(),
        },
    ]
});

/// Estimates difficulty level based on the concept keyword and language.
/// Returns "beginner", "intermediate", or "advanced".
pub fn estimate_difficulty(concept: &str, _language: &str) -> &'static str {
    let concept_lower = concept.to_lowercase();

    // Advanced concepts
    let advanced_keywords = [
        "concurrency", "async", "await", "mutex", "lock", "thread",
        "unsafe", "raw pointer", "ffi", "macro", "trait bound",
        "lifetime", "borrow", "atomic", "channel", "select",
        "generic", "template specialization", "move semantics",
        "smart pointer", "virtual inheritance", "metaprogramming",
        "coroutine", "future", "promise", "rwlock",
    ];
    for kw in &advanced_keywords {
        if concept_lower.contains(kw) {
            return "advanced";
        }
    }

    // Intermediate concepts
    let intermediate_keywords = [
        "trait", "impl", "struct method", "enum", "pattern match",
        "iterator", "closure", "map", "filter", "fold",
        "interface", "generics", "error handling", "option",
        "result", "pointer", "reference", "class", "inheritance",
        "polymorphism", "operator over", "slice", "module",
        "package", "context", "defer", "goroutine",
    ];
    for kw in &intermediate_keywords {
        if concept_lower.contains(kw) {
            return "intermediate";
        }
    }

    // Beginner concepts (default)
    let beginner_keywords = [
        "variable", "print", "function", "loop", "if", "for", "while",
        "string", "array", "vector", "list", "dict", "map",
        "hello", "input", "output", "type", "const", "let",
        "basic", "introduction", "getting started", "install",
    ];
    for kw in &beginner_keywords {
        if concept_lower.contains(kw) {
            return "beginner";
        }
    }

    // Default to intermediate if no keyword matches
    "intermediate"
}

/// File info returned by GitHub repository directory listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoFileInfo {
    pub name: String,
    pub path: String,
    pub file_type: String,
}

/// Language-to-file-extension mapping for filtering repository files.
pub fn language_extensions(language: &str) -> &'static [&'static str] {
    match language {
        "python" => &[".py"],
        "rust" => &[".rs"],
        "go" => &[".go"],
        "cpp" => &[".cpp", ".cxx", ".h", ".hpp", ".cc"],
        _ => &[],
    }
}

/// Determine language from file extension.
pub fn language_from_extension(path: &str) -> Option<&'static str> {
    if path.ends_with(".py") {
        Some("python")
    } else if path.ends_with(".rs") {
        Some("rust")
    } else if path.ends_with(".go") {
        Some("go")
    } else if path.ends_with(".cpp") || path.ends_with(".cxx") || path.ends_with(".cc") || path.ends_with(".hpp") {
        Some("cpp")
    } else {
        None
    }
}