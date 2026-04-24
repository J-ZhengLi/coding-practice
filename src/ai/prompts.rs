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

/// System prompt for the evaluate step (AI-06, SCORE-01).
///
/// Instructs the AI to compare user code against the original solution,
/// scoring STRUCTURAL COMPLETENESS (not functional correctness) per SCORE-06.
/// Returns structured JSON matching the EvaluationResult schema.
pub const EVALUATE_SYSTEM_PROMPT: &str = r#"You are a code evaluation assistant for programming exercises. Compare the user's implementation against the expected original code and provide a structural completeness score from 0 to 100. The score measures STRUCTURAL COMPLETENESS (not functional correctness) - how much of the expected code structure is present and correctly placed. Respond ONLY with valid JSON matching this schema: {"score": <number 0-100>, "letter_grade": "<A|B|C|D|F>", "is_partial": <boolean>, "feedback": {"strengths": ["<string>"], "improvements": ["<string>"], "summary": "<string>"}}.

Scoring criteria:
- 90-100 (A): Nearly complete structural match; all key components present and correctly structured
- 80-89 (B): Most structure present; minor omissions or slight misplacement
- 70-79 (C): Significant structure present but notable gaps or errors
- 60-69 (D): Partial structure; major sections missing or incorrectly structured
- 0-59 (F): Minimal structure; most expected components absent

Feedback requirements:
- strengths: 2-4 specific things the user did well (per SCORE-10)
- improvements: 2-4 specific areas for improvement (per SCORE-09)
- summary: 1-2 sentence overall assessment (per SCORE-08)

If TODO markers remain in the user's code, set is_partial to true."#;

/// User prompt template for the evaluate step.
///
/// Provides the exercise context, original code, user's submission, and TODO comment.
pub const EVALUATE_USER_PROMPT: &str = r#"Evaluate the following {language} code submission for the exercise "{title}".

## Exercise Description
{description}

## Original Code (Expected Solution)
```{language}
{original_code}
```

## User's Submission
```{language}
{user_code}
```

## TODO Comment
{todo_comment}

Score the structural completeness of the user's implementation. Check if the code structure matches the expected solution. Provide specific strengths and specific areas for improvement."#;

/// Fills the EVALUATE_SYSTEM_PROMPT template (no placeholders in system prompt, but kept for consistency).
pub fn format_evaluate_system_prompt() -> String {
    EVALUATE_SYSTEM_PROMPT.to_string()
}

/// Fills the EVALUATE_USER_PROMPT template with exercise details.
pub fn format_evaluate_user_prompt(
    language: &str,
    title: &str,
    description: &str,
    original_code: &str,
    user_code: &str,
    todo_comment: &str,
) -> String {
    EVALUATE_USER_PROMPT
        .replace("{language}", language)
        .replace("{title}", title)
        .replace("{description}", description)
        .replace("{original_code}", original_code)
        .replace("{user_code}", user_code)
        .replace("{todo_comment}", todo_comment)
}