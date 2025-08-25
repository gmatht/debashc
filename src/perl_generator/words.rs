use crate::ast::*;
use super::PerlGenerator;

impl PerlGenerator {
    pub fn word_to_perl(&mut self, word: &Word) -> String {
        match word {
            Word::Literal(s) => {
                // Handle literal strings
                if s.contains("..") {
                    self.handle_range_expansion(s)
                } else if s.contains(',') {
                    self.handle_comma_expansion(s)
                } else {
                    s.clone()
                }
            },
            Word::ParameterExpansion(pe) => self.generate_parameter_expansion(pe),
            Word::Array(name, elements) => {
                let elements_str = elements.iter()
                    .map(|e| format!("'{}'", e.replace("'", "\\'")))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("@{} = ({});", name, elements_str)
            },
            Word::StringInterpolation(interp) => self.convert_string_interpolation_to_perl(interp),
            Word::Arithmetic(expr) => self.convert_arithmetic_to_perl(&expr.expression),
            Word::BraceExpansion(expansion) => self.handle_brace_expansion(expansion),
            _ => format!("{:?}", word)
        }
    }

    pub fn word_to_perl_for_test(&self, word: &Word) -> String {
        match word {
            Word::Literal(s) => s.clone(),
            Word::ParameterExpansion(pe) => self.generate_parameter_expansion(pe),
            _ => format!("{:?}", word)
        }
    }

    // Helper methods
    fn handle_range_expansion(&self, s: &str) -> String {
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

    fn handle_comma_expansion(&self, s: &str) -> String {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() > 1 {
            parts.join(" ")
        } else {
            s.to_string()
        }
    }

    fn handle_brace_expansion(&mut self, expansion: &BraceExpansion) -> String {
        if expansion.items.len() == 1 {
            self.word_to_perl(&self.brace_item_to_word(&expansion.items[0]))
        } else {
            let items: Vec<String> = expansion.items.iter()
                .map(|item| self.word_to_perl(&self.brace_item_to_word(item)))
                .collect();
            items.join(" ")
        }
    }

    fn brace_item_to_word(&self, item: &BraceItem) -> Word {
        match item {
            BraceItem::Literal(s) => Word::Literal(s.clone()),
            BraceItem::Range(range) => Word::Literal(format!("{}..{}", range.start, range.end)),
            BraceItem::Sequence(seq) => Word::Literal(seq.join(" ")),
        }
    }

    fn convert_string_interpolation_to_perl(&self, interp: &StringInterpolation) -> String {
        // Basic string interpolation - just return the parts joined together
        interp.parts.iter().map(|part| format!("{:?}", part)).collect::<Vec<_>>().join("")
    }

    fn convert_arithmetic_to_perl(&self, expr: &str) -> String {
        // Basic arithmetic conversion
        expr.replace("**", "**")
            .replace("==", "==")
            .replace("!=", "!=")
            .replace("<=", "<=")
            .replace(">=", ">=")
    }
}
