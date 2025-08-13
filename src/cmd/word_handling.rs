use crate::ast::*;

pub trait WordHandler {
    fn word_to_perl(&mut self, word: &Word) -> String;
    fn word_to_perl_for_test(&self, word: &Word) -> String;
}

impl<T: WordHandler> WordHandler for T {
    fn word_to_perl(&mut self, word: &Word) -> String {
        // Placeholder implementation
        format!("word_{:?}", word)
    }
    
    fn word_to_perl_for_test(&self, word: &Word) -> String {
        // Placeholder implementation
        format!("test_word_{:?}", word)
    }
}
