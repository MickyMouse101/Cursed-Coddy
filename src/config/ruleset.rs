use crate::config::{Difficulty, Language, LessonType};

pub struct Ruleset;

impl Ruleset {
    pub fn generate_prompt(
        language: Language,
        difficulty: Difficulty,
        lesson_type: LessonType,
        topic: &str,
    ) -> String {
        let concept_count = lesson_type.concept_count();
        let exercise_count = lesson_type.exercise_count();

        format!(
            r#"You are a coding education assistant similar to Codecademy and Coddy. Generate an educational lesson following these rules:

LANGUAGE: {}
DIFFICULTY: {}
LESSON TYPE: {}
TOPIC: {}

TEACHING STYLE REQUIREMENTS:
1. **Concept Introduction**: Provide a clear, beginner-friendly explanation of what the concept is (5-7 sentences). Include:
   - What the concept is and why it exists
   - How it differs from other languages (if applicable)
   - Why this design choice was made (the reasoning behind it)
   - Common misconceptions or differences from languages like JavaScript, Python, C++, etc.
   - Real-world context: when and why you'd use this feature
2. **Step-by-Step Explanation**: Break down how the concept works in simple steps (4-6 steps). Each step should explain:
   - What happens at that step
   - Why it works that way
   - How it differs from similar concepts in other languages (if applicable)
3. **Code Examples**: **CRITICAL: You MUST provide at least 2 code examples** (2-3 total). Each example should have detailed line-by-line explanations. Include:
   - What each line does
   - Why it's written that way
   - What would happen if you tried to do it differently
   - Comparisons to how you'd do it in other languages (if relevant)
4. **Syntax Guide**: Explain the syntax clearly, showing how to write it with examples. Include:
   - The exact syntax rules
   - What each part means
   - Common variations
   - What happens if you omit parts (e.g., what if you forget `mut`?)
5. **Common Patterns**: Show 2-3 common use cases or patterns for this concept. Explain:
   - When to use each pattern
   - Why that pattern is preferred
   - What problems it solves
6. **Guided Exercise**: Create {} exercise(s) with:
   - Clear step-by-step instructions that explain WHAT to do and HOW to do it
   - **CRITICAL FOR BEGINNER EXERCISES**: For basic concepts like variable declaration, use hardcoded values. DO NOT require input reading unless the topic explicitly teaches input/output. Keep it simple for beginners.
   - **CRITICAL**: DO NOT create exercises that require compilation errors to produce expected output. If you want to teach about errors, explain them in the concept/step-by-step sections, but exercises should produce valid, compilable code that runs successfully.
   - **CRITICAL**: If the exercise requires reading input, EXPLICITLY state this in the description and explain HOW to read input for the language with code examples
   - **CRITICAL**: If the exercise requires output, EXPLICITLY state this in the description
   - Detailed hints on how to approach it (be specific, not vague)
   - **Example input/output**: ALWAYS provide these fields:
     * "example_input": Show what input the program should read (use empty string "" if no input needed - this is preferred for beginner exercises)
     * "example_output": Show what the program should output/print (ALWAYS include this - even if just showing variable values or function results)
     * For variable declaration exercises: Use hardcoded values and show what the code should output (e.g., print the variable value). DO NOT require input reading.
     * For exercises requiring input: clearly state "Your program should read input from stdin" and show example input values with code examples
   - **CRITICAL: Test cases are REQUIRED** - You MUST include at least 2-3 test cases for EVERY exercise:
     * Test cases MUST align with the exercise description and hints
     * Test cases MUST validate what the exercise asks the student to do
     * If the exercise asks to print something, test cases should check that exact output
     * If the exercise asks to calculate something, test cases should verify the calculation
     * **IMPORTANT**: If test cases have different outputs but no input, this is an error. Either:
       - Use the same expected output for all test cases (for exercises without input), OR
       - Provide input values for each test case (for exercises with input)
   - **IMPORTANT: You MUST include at least {} exercise(s) in the "exercises" array, and EACH exercise MUST have test_cases with at least 2-3 test cases**

EDUCATIONAL FOCUS:
- Teach HOW to write the code, not just what to write
- Explain WHY certain syntax is used and the reasoning behind design choices
- Compare and contrast with other languages (JavaScript, Python, C++, etc.) to help learners understand differences
- Show common mistakes and how to avoid them, including what happens if you try to do things the "other language way"
- Make it practical and applicable to real coding
- For {} lessons, focus on {} core concept(s)
- Match the complexity to {} difficulty level
- **SPECIAL FOR RUST**: When explaining concepts like immutability, ownership, borrowing, etc.:
  * Explain how Rust differs from other languages (e.g., in JavaScript/Python, variables are mutable by default)
  * Explain WHY Rust made these design choices (memory safety, preventing bugs, etc.)
  * Show what happens if you try to change an immutable variable (the compiler error)
  * Compare to how you'd do it in other languages
  * Explain the benefits and trade-offs
- **SPECIAL FOR COMPILATION TOPICS**: 
  * Explain the compilation/build process step-by-step
  * Show actual command-line examples (e.g., "g++ -o program program.cpp")
  * Explain what each flag/option does
  * Show how to run the compiled program
  * For C++: Cover both direct g++ compilation and CMake basics
  * For Rust: Cover both rustc direct compilation and Cargo project management
  * For JavaScript: Explain Node.js execution, no compilation needed but show how to run scripts

OUTPUT FORMAT (JSON):
{{
  "concept": "Clear, detailed explanation (5-7 sentences) of what the concept is, why it exists, how it differs from other languages, and the reasoning behind the design choice. Include comparisons to JavaScript, Python, C++, etc. when relevant. For Rust immutability, explain that `let` without `mut` creates an immutable variable that CANNOT be changed after assignment (unlike JavaScript/Python where variables are mutable by default), and explain WHY Rust made this design choice (memory safety, preventing bugs, etc.).",
  "step_by_step": [
    "Step 1: Detailed explanation including what happens and why",
    "Step 2: Detailed explanation including what happens and why",
    "Step 3: Detailed explanation including what happens and why"
  ],
  "code_examples": [
    {{
      "code": "example code here",
      "explanation": "Detailed line-by-line explanation of what this code does, why it's written that way, what would happen if done differently, and comparisons to other languages when relevant. For Rust, show what compiler error you get if you try to change an immutable variable."
    }},
    {{
      "code": "another example code here",
      "explanation": "Detailed explanation of this second example, showing a different approach or variation."
    }}
  ],
   **CRITICAL: The "code_examples" array MUST contain at least 2 examples. Do not provide just 1 example.**
  "syntax_guide": "Detailed explanation of the syntax with examples. Include what each part means, what happens if you omit parts, and comparisons to other languages. For Rust immutability, explain that `let` without `mut` creates an immutable variable that CANNOT be changed (unlike JavaScript/Python where variables are mutable by default), and show what error you get if you try.",
  "common_patterns": [
    "Pattern 1: Detailed description explaining when to use it, why it's preferred, and what problems it solves",
    "Pattern 2: Detailed description explaining when to use it, why it's preferred, and what problems it solves"
  ],
  "exercises": [
    {{
      "title": "Exercise title",
      "description": "Detailed step-by-step instructions",
      "hints": ["Hint 1", "Hint 2"],
      "example_input": "example input (use empty string \"\" if no input needed, but ALWAYS include this field)",
      "example_output": "expected output (show what the code should produce/print, ALWAYS include this field)",
      "test_cases": [
        {{"input": "...", "output": "..."}},
        {{"input": "...", "output": "..."}},
        {{"input": "...", "output": "..."}}
      ]
       **CRITICAL: The "test_cases" array MUST contain at least 2-3 test cases. Test cases MUST validate the exercise requirements and align with the description and hints.**
    }}
  ]
}}

       **CRITICAL: The "exercises" array is REQUIRED and must contain at least {} exercise(s). Do not omit this field.**

       **IMPORTANT JSON FORMATTING RULES:**
       - Output ONLY valid JSON - no markdown code fences, no explanatory text before or after
       - Ensure all strings are properly escaped (use \" for quotes inside strings)
       - Ensure all brackets and braces are properly closed
       - Do not include trailing commas
       - Make sure the JSON is complete and valid before finishing

       Generate the educational lesson now. Output ONLY the JSON object, nothing else:"#,
            language.display_name(),
            difficulty.display_name(),
            lesson_type.display_name(),
            topic,
            exercise_count,
            exercise_count, // Duplicate for emphasis
            lesson_type.display_name(),
            concept_count,
            difficulty.display_name(),
            exercise_count // Final emphasis
        )
    }
}
