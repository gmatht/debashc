use crate::ast::*;

pub trait PipelineHandler {
    fn generate_pipeline(&mut self, pipeline: &Pipeline) -> String;
    fn command_to_string(&mut self, command: &Command) -> String;
    fn pipeline_counter(&mut self) -> &mut usize;
    fn needs_file_find(&mut self) -> &mut bool;
    fn word_to_perl(&mut self, word: &Word) -> String;
    fn convert_string_interpolation_to_perl(&self, interp: &StringInterpolation) -> String;
    fn convert_glob_to_regex(&self, pattern: &str) -> String;
    fn indent(&self) -> String;
}

impl<T: PipelineHandler> PipelineHandler for T {
    fn generate_pipeline(&mut self, _pipeline: &Pipeline) -> String {
        // Placeholder implementation - this will be expanded later
        let mut output = String::new();
        output.push_str("// Pipeline generation placeholder\n");
        output
    }

    fn command_to_string(&mut self, command: &Command) -> String {
        // Placeholder implementation
        format!("command_{:?}", command)
    }
    
    fn pipeline_counter(&mut self) -> &mut usize {
        // Placeholder implementation - this should be overridden by concrete implementations
        // that store the counter in the struct itself
        panic!("pipeline_counter() must be implemented by concrete types")
    }
    
    fn needs_file_find(&mut self) -> &mut bool {
        // Placeholder implementation - this should be overridden by concrete implementations
        // that store the flag in the struct itself
        panic!("needs_file_find() must be implemented by concrete types")
    }
    
    fn word_to_perl(&mut self, word: &Word) -> String {
        // Placeholder implementation
        format!("word_{:?}", word)
    }
    
    fn convert_string_interpolation_to_perl(&self, interp: &StringInterpolation) -> String {
        // Placeholder implementation
        format!("interpolation_{}", interp.parts.len())
    }
    
    fn convert_glob_to_regex(&self, pattern: &str) -> String {
        // Placeholder implementation
        format!("regex_{}", pattern)
    }
    
    fn indent(&self) -> String {
        // Placeholder implementation
        "    ".to_string()
    }
}
