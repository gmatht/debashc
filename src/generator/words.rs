use crate::ast::*;
use super::Generator;

pub fn word_to_perl_impl(generator: &mut Generator, word: &Word) -> String {
    match word {
        Word::Literal(s) => {
            // Handle literal strings
            if s.contains("..") {
                generator.handle_range_expansion(s)
            } else if s.contains(',') {
                generator.handle_comma_expansion(s)
            } else {
                s.clone()
            }
        },
        Word::ParameterExpansion(pe) => generator.generate_parameter_expansion(pe),
        Word::Array(name, elements) => {
            let elements_str = elements.iter()
                .map(|e| format!("'{}'", e.replace("'", "\\'")))
                .collect::<Vec<_>>()
                .join(", ");
            format!("@{} = ({});", name, elements_str)
        },
        Word::StringInterpolation(interp) => generator.convert_string_interpolation_to_perl(interp),
        Word::Arithmetic(expr) => generator.convert_arithmetic_to_perl(&expr.expression),
        Word::BraceExpansion(expansion) => generator.handle_brace_expansion(expansion),
        _ => format!("{:?}", word)
    }
}

pub fn word_to_perl_for_test_impl(generator: &mut Generator, word: &Word) -> String {
    match word {
        Word::Literal(s) => s.clone(),
        Word::ParameterExpansion(pe) => generator.generate_parameter_expansion(pe),
        _ => format!("{:?}", word)
    }
}

// Helper methods
pub fn handle_range_expansion_impl(generator: &Generator, s: &str) -> String {
    let parts: Vec<&str> = s.split("..").collect();
    if parts.len() == 2 {
        if let (Ok(start), Ok(end)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>()) {
            let values: Vec<String> = (start..=end)
                .map(|i| i.to_string())
                .collect();
            // Format as Perl array: (1, 2, 3, 4, 5)
            format!("({})", values.join(", "))
        } else {
            s.to_string()
        }
    } else {
        s.to_string()
    }
}

pub fn handle_comma_expansion_impl(generator: &Generator, s: &str) -> String {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() > 1 {
        parts.join(" ")
    } else {
        s.to_string()
    }
}

pub fn handle_brace_expansion_impl(generator: &mut Generator, expansion: &BraceExpansion) -> String {
    if expansion.items.len() == 1 {
        generator.word_to_perl(&generator.brace_item_to_word(&expansion.items[0]))
    } else {
        let items: Vec<String> = expansion.items.iter()
            .map(|item| generator.word_to_perl(&generator.brace_item_to_word(item)))
            .collect();
        items.join(" ")
    }
}

pub fn brace_item_to_word_impl(generator: &Generator, item: &BraceItem) -> Word {
    match item {
        BraceItem::Literal(s) => Word::Literal(s.clone()),
        BraceItem::Range(range) => Word::Literal(format!("{}..{}", range.start, range.end)),
        BraceItem::Sequence(seq) => Word::Literal(seq.join(" ")),
    }
}

pub fn convert_string_interpolation_to_perl_impl(generator: &Generator, interp: &StringInterpolation) -> String {
    // Convert string interpolation to proper Perl string concatenation
    let parts: Vec<String> = interp.parts.iter().map(|part| {
        match part {
            StringPart::Literal(s) => format!("\"{}\"", generator.escape_perl_string(s)),
            StringPart::Variable(var) => {
                // Check if this is a shell positional parameter ($1, $2, etc.)
                if var.chars().all(|c| c.is_digit(10)) {
                    // Convert $1 to $_[0], $2 to $_[1], etc.
                    let index = var.parse::<usize>().unwrap_or(0);
                    format!("$_[{}]", index - 1) // Perl arrays are 0-indexed
                } else {
                    // Regular variable
                    format!("${}", var)
                }
            },
            StringPart::MapAccess(map_name, key) => {
                if map_name == "map" {
                    format!("$map{{{}}}", key)
                } else {
                    format!("${}{{{}}}", map_name, key)
                }
            }
            _ => {
                // Handle other StringPart variants by converting them to debug format for now
                format!("{:?}", part)
            }
        }
    }).collect();
    
    // Join with Perl concatenation operator
    parts.join(" . ")
}

pub fn convert_arithmetic_to_perl_impl(generator: &Generator, expr: &str) -> String {
    // Convert shell arithmetic expression to Perl syntax
    let mut result = expr.to_string();
    
    // Convert shell variables to Perl variables (e.g., i -> $i)
    // This is a simple regex-based approach - in practice, the parser should handle this better
    
    // Split the expression into parts and convert each part
    let parts: Vec<&str> = result.split_whitespace().collect();
    let converted_parts: Vec<String> = parts.iter().map(|part| {
        // Check if this part looks like a variable (starts with a letter and contains only alphanumeric chars)
        if part.chars().next().map_or(false, |c| c.is_alphabetic()) && 
           part.chars().all(|c| c.is_alphanumeric() || c == '_') {
            // This looks like a variable, prefix with $
            format!("${}", part)
        } else {
            // This is an operator, number, or already formatted variable
            part.to_string()
        }
    }).collect();
    
    // Rejoin the parts
    converted_parts.join(" ")
}
