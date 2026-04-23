use std::collections::HashMap;

/// System prompt template for the analyze step.
///
/// Instructs the AI to identify exercise-worthy sections in source code
/// and return structured JSON matching the AnalysisResult schema.
pub const ANALYZE_SYSTEM_PROMPT: &str = r#"You are a coding exercise analyzer for {language} at {difficulty} level. Analyze the provided source code and identify sections suitable for exercise generation. For each section, provide the start line, end line, difficulty level, the programming concept it demonstrates, and why it makes a good exercise. Respond ONLY with valid JSON matching this schema: {"sections": [{"start_line": <number>, "end_line": <number>, "difficulty": "beginner|intermediate|advanced", "concept": "<string>", "reason": "<string>"}]}"#;

/// User prompt template for the analyze step.
///
/// Provides the source code and analysis parameters.
pub const ANALYZE_USER_PROMPT: &str = r#"Analyze the following {language} source code and identify up to {max_sections} sections suitable for coding exercises at the {difficulty} level:

```{language}
{code}
```"#;

/// System prompt template for the generate step.
///
/// Instructs the AI to create a TODO-based exercise from an identified
/// code section, returning structured JSON matching the ExerciseResult schema.
pub const GENERATE_SYSTEM_PROMPT: &str = r#"You are a coding exercise generator for {language}. Given an identified code section, create a TODO-based exercise. The TODO comment must use {comment_syntax} comment syntax. Replace the identified code section with a TODO comment that describes what needs to be implemented. Include a descriptive title and a 2-3 sentence explanation. Respond ONLY with valid JSON matching this schema: {"title": "<string>", "description": "<string>", "todo_comment": "<string>", "difficulty": "beginner|intermediate|advanced", "language": "<string>", "original_code": "<string>", "exercise_code": "<string>"}"#;

/// User prompt template for the generate step.
///
/// Provides the identified section, concept, and full source code for context.
pub const GENERATE_USER_PROMPT: &str = r#"Create a {difficulty} level exercise for {language} from the following code section:

Concept: {concept}
Reason: {reason}

Original code (lines {start_line}-{end_line}):
```{language}
{original_code}
```

Full source for context:
```{language}
{full_code}
```"#;

/// Language-to-comment-syntax mapping for TODO markers.
///
/// Maps programming language names to their single-line comment prefix,
/// per D-06 (language-specific comment syntax).
fn comment_syntax_map() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("python", "#");
    map.insert("rust", "//");
    map.insert("go", "//");
    map.insert("c++", "//");
    map.insert("cpp", "//");
    map.insert("c", "//");
    map.insert("javascript", "//");
    map.insert("typescript", "//");
    map.insert("java", "//");
    map
}

/// Returns the appropriate comment syntax prefix for a given programming language.
///
/// Defaults to `//` for unknown languages.
pub fn get_comment_syntax(language: &str) -> &'static str {
    comment_syntax_map()
        .get(language.to_lowercase().as_str())
        .copied()
        .unwrap_or("//")
}

/// Fills the ANALYZE_SYSTEM_PROMPT template with language and difficulty.
pub fn format_analyze_system_prompt(language: &str, difficulty: &str) -> String {
    ANALYZE_SYSTEM_PROMPT
        .replace("{language}", language)
        .replace("{difficulty}", difficulty)
}

/// Fills the ANALYZE_USER_PROMPT template with language, max_sections, and code.
pub fn format_analyze_user_prompt(language: &str, max_sections: usize, difficulty: &str, code: &str) -> String {
    ANALYZE_USER_PROMPT
        .replace("{language}", language)
        .replace("{max_sections}", &max_sections.to_string())
        .replace("{difficulty}", difficulty)
        .replace("{code}", code)
}

/// Fills the GENERATE_SYSTEM_PROMPT template with language and comment syntax.
pub fn format_generate_system_prompt(language: &str) -> String {
    GENERATE_SYSTEM_PROMPT
        .replace("{language}", language)
        .replace("{comment_syntax}", get_comment_syntax(language))
}

/// Fills the GENERATE_USER_PROMPT template with all parameters.
pub fn format_generate_user_prompt(
    difficulty: &str,
    language: &str,
    concept: &str,
    reason: &str,
    start_line: usize,
    end_line: usize,
    original_code: &str,
    full_code: &str,
) -> String {
    GENERATE_USER_PROMPT
        .replace("{difficulty}", difficulty)
        .replace("{language}", language)
        .replace("{concept}", concept)
        .replace("{reason}", reason)
        .replace("{start_line}", &start_line.to_string())
        .replace("{end_line}", &end_line.to_string())
        .replace("{original_code}", original_code)
        .replace("{full_code}", full_code)
}