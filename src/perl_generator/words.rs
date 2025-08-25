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
            values.join(" ")
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
    // Basic string interpolation - just return the parts joined together
    interp.parts.iter().map(|part| format!("{:?}", part)).collect::<Vec<_>>().join("")
}

pub fn convert_arithmetic_to_perl_impl(generator: &Generator, expr: &str) -> String {
    // Basic arithmetic conversion
    expr.replace("**", "**")
        .replace("==", "==")
        .replace("!=", "!=")
        .replace("<=", "<=")
        .replace(">=", ">=")
}
