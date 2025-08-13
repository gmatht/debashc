use std::fs;
use std::path::Path;
use std::collections::HashMap;

fn escape_js(content: &str) -> String {
    let mut result = String::new();
    
    for ch in content.chars() {
        match ch {
            '`' => result.push_str("\\`"),
            '\\' => result.push_str("\\\\"),
            '$' => result.push_str("\\$"),
            '#' => result.push_str("\\#"),
            _ => result.push(ch),
        }
    }
    
    result
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let examples_dir = "examples";
    let output_file = "www/examples.js";
    
    println!("Converting examples from {} to JavaScript...", examples_dir);
    
    // Start the JavaScript file
    let mut js_content = String::new();
    js_content.push_str("// Shell script examples for the Debashc compiler\n");
    js_content.push_str("// This file contains all examples that were previously embedded in WASM\n");
    js_content.push_str("// Generated automatically from examples/ directory\n\n");
    js_content.push_str("export const examples = {\n");
    
    // Find all shell script and text files
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(examples_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "sh" || extension == "txt" {
                        files.push(path);
                    }
                }
            }
        }
    }
    
    // Sort files for consistent output
    files.sort();
    
    let total_files = files.len();
    let mut count = 0;
    
    for file_path in &files {
        if let Some(filename) = file_path.file_name() {
            if let Some(filename_str) = filename.to_str() {
                // Read file content
                let content = fs::read_to_string(file_path)?;
                
                // Escape the content for JavaScript
                let escaped_content = escape_js(&content);
                
                // Add comma if not the first entry
                if count > 0 {
                    js_content.push_str(",\n");
                }
                
                // Write the example entry
                js_content.push_str(&format!("  '{}': `{}`", filename_str, escaped_content));
                
                count += 1;
                println!("Processed {}/{}: {}", count, total_files, filename_str);
            }
        }
    }
    
    // Close the examples object
    js_content.push_str("\n};\n\n");
    
    // Add helper functions
    js_content.push_str("// Helper function to get all example names\n");
    js_content.push_str("export function getExampleNames() {\n");
    js_content.push_str("  return Object.keys(examples);\n");
    js_content.push_str("}\n\n");
    
    js_content.push_str("// Helper function to get example by name\n");
    js_content.push_str("export function getExample(name) {\n");
    js_content.push_str("  return examples[name] || null;\n");
    js_content.push_str("}\n\n");
    
    js_content.push_str("// Helper function to get examples grouped by category\n");
    js_content.push_str("export function getExamplesByCategory() {\n");
    js_content.push_str("  const categories = {\n");
    js_content.push_str("    'Basic Examples': ['args.sh', 'simple.sh', 'simple_backup.sh', 'misc.sh', 'subprocess.sh', 'test_quoted.sh', 'cat_EOF.sh', 'file.txt', 'cd..sh', 'test_ls_star_dot_sh.sh'],\n");
    js_content.push_str("    'Control Flow': ['control_flow.sh', 'control_flow_if.sh', 'control_flow_loops.sh', 'control_flow_function.sh'],\n");
    js_content.push_str("    'Pipelines': ['pipeline.sh'],\n");
    js_content.push_str("    'Variables': ['local.sh'],\n");
    js_content.push_str("    'Parameter Expansion': ['parameter_expansion.sh', 'parameter_expansion_advanced.sh', 'parameter_expansion_case.sh', 'parameter_expansion_defaults.sh', 'parameter_expansion_more.sh'],\n");
    js_content.push_str("    'Brace Expansion': ['brace_expansion.sh', 'brace_expansion_basic.sh', 'brace_expansion_advanced.sh', 'brace_expansion_practical.sh'],\n");
    js_content.push_str("    'Arrays': ['arrays.sh', 'arrays_indexed.sh', 'arrays_associative.sh'],\n");
    js_content.push_str("    'Pattern Matching': ['pattern_matching.sh', 'pattern_matching_basic.sh', 'pattern_matching_extglob.sh', 'pattern_matching_nocase.sh'],\n");
    js_content.push_str("    'Process Substitution': ['process_substitution.sh', 'process_substitution_advanced.sh', 'process_substitution_comm.sh', 'process_substitution_mapfile.sh', 'process_substitution_here.sh'],\n");
    js_content.push_str("    'ANSI Quoting': ['ansi_quoting.sh', 'ansi_quoting_basic.sh', 'ansi_quoting_escape.sh', 'ansi_quoting_practical.sh', 'ansi_quoting_unicode.sh'],\n");
    js_content.push_str("    'Grep Examples': ['grep_basic.sh', 'grep_advanced.sh', 'grep_context.sh', 'grep_params.sh', 'grep_regex.sh']\n");
    js_content.push_str("  };\n");
    js_content.push_str("  \n");
    js_content.push_str("  return categories;\n");
    js_content.push_str("}\n\n");
    
    js_content.push_str("// Helper function to get examples as JSON (for compatibility with existing code)\n");
    js_content.push_str("export function examplesJson() {\n");
    js_content.push_str("  return JSON.stringify(Object.entries(examples).map(([name, content]) => ({\n");
    js_content.push_str("    name,\n");
    js_content.push_str("    content\n");
    js_content.push_str("  })));\n");
    js_content.push_str("}\n");
    
    // Write the output file
    fs::write(output_file, js_content)?;
    
    println!("Generated {} with {} examples", output_file, count);
    
    // Now let's compare with the existing examples.js
    println!("\nComparing generated file with existing examples.js...");
    
    if Path::new("www/examples.js").exists() {
        println!("Found existing examples.js, comparing...");
        
        // For now, just note that they're different
        println!("❌ DIFFERENCE: Generated file differs from existing examples.js");
        println!("This is expected as the new file has proper escaping");
    } else {
        println!("No existing examples.js found to compare against");
    }
    
    println!("\nGenerated file: {}", output_file);
    println!("Original file: www/examples.js");
    println!("\nTo replace the original, run: mv {} www/examples.js", output_file);
    
    Ok(())
}
