use crate::ast::*;
use super::Generator;

pub fn generate_parameter_expansion_impl(_generator: &mut Generator, pe: &ParameterExpansion) -> String {
    match &pe.operator {
        ParameterExpansionOperator::None => {
            // ${var} - just the variable
            format!("${{{}}}", pe.variable)
        }
        ParameterExpansionOperator::DefaultValue(default) => {
            // ${var:-default} - use default if var is empty
            format!("defined(${{{}}}) && ${{{}}} ne '' ? ${{{}}} : {}", 
                   pe.variable, pe.variable, pe.variable, default)
        }
        ParameterExpansionOperator::AssignDefault(default) => {
            // ${var:=default} - assign default if var is empty
            format!("defined(${{{}}}) && ${{{}}} ne '' ? ${{{}}} : do {{ ${{{}}} = {}; ${{{}}} }}", 
                   pe.variable, pe.variable, pe.variable, pe.variable, default, pe.variable)
        }
        ParameterExpansionOperator::ErrorIfUnset(error) => {
            // ${var:?error} - error if var is empty
            format!("defined(${{{}}}) && ${{{}}} ne '' ? ${{{}}} : die({})", 
                   pe.variable, pe.variable, pe.variable, error)
        }
        ParameterExpansionOperator::RemoveShortestSuffix(pattern) => {
            // ${var%suffix} - remove shortest suffix
            format!("${{{}}} =~ s/{}$//r", pe.variable, escape_regex_pattern(pattern))
        }
        ParameterExpansionOperator::RemoveLongestSuffix(pattern) => {
            // ${var%%suffix} - remove longest suffix
            format!("${{{}}} =~ s/{}$//gr", pe.variable, escape_regex_pattern(pattern))
        }
        ParameterExpansionOperator::RemoveShortestPrefix(pattern) => {
            // ${var#prefix} - remove shortest prefix
            format!("${{{}}} =~ s/^{}//r", pe.variable, escape_regex_pattern(pattern))
        }
        ParameterExpansionOperator::RemoveLongestPrefix(pattern) => {
            // ${var##prefix} - remove longest prefix
            format!("${{{}}} =~ s/^{}//gr", pe.variable, escape_regex_pattern(pattern))
        }
        ParameterExpansionOperator::SubstituteAll(pattern, replacement) => {
            // ${var//pattern/replacement} - substitute all occurrences
            format!("${{{}}} =~ s/{}/{}/gr", 
                   pe.variable, 
                   escape_regex_pattern(pattern),
                   escape_regex_replacement(replacement))
        }
        ParameterExpansionOperator::UppercaseAll => {
            // ${var^^} - uppercase all characters
            format!("uc(${{{}}})", pe.variable)
        }
        ParameterExpansionOperator::LowercaseAll => {
            // ${var,,} - lowercase all characters
            format!("lc(${{{}}})", pe.variable)
        }
        ParameterExpansionOperator::UppercaseFirst => {
            // ${var^} - uppercase first character
            format!("ucfirst(${{{}}})", pe.variable)
        }
        ParameterExpansionOperator::Basename => {
            // ${var##*/} - get basename
            format!("basename(${{{}}})", pe.variable)
        }
        ParameterExpansionOperator::Dirname => {
            // ${var%/*} - get dirname
            format!("dirname(${{{}}})", pe.variable)
        }
        ParameterExpansionOperator::ArraySlice(offset, length) => {
            // ${var:offset:length} - array slice
            if let Some(length_str) = length {
                format!("@${{{}}}[{}..{}]", pe.variable, offset, length_str)
            } else {
                format!("@${{{}}}[{}..]", pe.variable, offset)
            }
        }
    }
}

// Helper methods for regex escaping
fn escape_regex_pattern(pattern: &str) -> String {
    // Escape special regex characters in the pattern
    pattern.replace("\\", "\\\\")
           .replace(".", "\\.")
           .replace("+", "\\+")
           .replace("*", "\\*")
           .replace("?", "\\?")
           .replace("^", "\\^")
           .replace("$", "\\$")
           .replace("[", "\\[")
           .replace("]", "\\]")
           .replace("(", "\\(")
           .replace(")", "\\)")
           .replace("|", "\\|")
}

fn escape_regex_replacement(replacement: &str) -> String {
    // Escape special regex characters in the replacement string
    replacement.replace("\\", "\\\\")
               .replace("$", "\\$")
               .replace("&", "\\&")
               .replace("`", "\\`")
               .replace("'", "\\'")
}
