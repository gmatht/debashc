use logos::Logos;
use thiserror::Error;
use std::cmp::Ordering;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("elif")]
    Elif,
    #[token("fi")]
    Fi,
    #[token("while")]
    While,
    #[token("do")]
    Do,
    #[token("done")]
    Done,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("function")]
    Function,
    #[token("case")]
    Case,
    #[token("esac")]
    Esac,
    #[token("select")]
    Select,
    #[token("until")]
    Until,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("return")]
    Return,
    #[token("exit")]
    Exit,
    #[token("export")]
    Export,
    #[token("readonly")]
    Readonly,
    #[token("local")]
    Local,
    #[token("declare")]
    Declare,
    #[token("typeset")]
    Typeset,
    #[token("unset")]
    Unset,
    #[token("shift")]
    Shift,
    #[token("set")]
    Set,
    #[token("eval")]
    Eval,
    #[token("exec")]
    Exec,
    #[token("source")]
    Source,
    // SourceDot removed - dots in filenames should be part of identifiers
    #[token("trap")]
    Trap,
    #[token("wait")]
    Wait,
    #[token("shopt")]
    Shopt,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("[")]
    TestBracket,
    #[token("]")]
    TestBracketClose,

    // Operators
    #[token("|")]
    Pipe,
    #[token("||")]
    Or,
    #[token("&")]
    Background,
    #[token("&&")]
    And,
    #[token(";")]
    Semicolon,
    #[token(";;")]
    DoubleSemicolon,
    #[token("..", priority = 3)]
    Range,
    #[token("(")]
    ParenOpen,
    #[token(")")]
    ParenClose,
    #[token("{")]
    BraceOpen,
    #[token("}")]
    BraceClose,
    #[token("=")]
    Assign,
    #[token("+=")]
    PlusAssign,
    #[token("-=")]
    MinusAssign,
    #[token("*=")]
    StarAssign,
    #[token("/=")]
    SlashAssign,
    #[token("%=")]
    PercentAssign,
    #[token("**=")]
    StarStarAssign,
    #[token("<<=")]
    LeftShiftAssign,
    #[token(">>=")]
    RightShiftAssign,
    #[token("&=")]
    AndAssign,
    #[token("^=")]
    CaretAssign,
    #[token("|=")]
    OrAssign,

    // Redirections
    #[token("<")]
    RedirectIn,
    #[token(">")]
    RedirectOut,
    #[token(">>")]
    RedirectAppend,
    #[token("<>")]
    RedirectInOut,
    #[token("<<")]
    Heredoc,
    #[token("<<-")]
    HeredocTabs,
    #[token("<<<")]
    HereString,
    #[token(">&")]
    RedirectOutErr,
    #[token("<&")]
    RedirectInErr,
    #[token(">|")]
    RedirectOutClobber,
    #[token("&>")]
    RedirectAll,
    #[token("&>>")]
    RedirectAllAppend,

    // Variables and expansions
    #[token("$")]
    Dollar,
    #[token("${")]
    DollarBrace,
    #[token("$(")]
    DollarParen,
    #[token("$#")]
    DollarHashSimple,
    #[token("$@")]
    DollarAtSimple,
    #[token("$*")]
    DollarStarSimple,
    // Backtick token not currently used
    #[token("`", priority = 1)]
    _Backtick, // Unused variant, prefixed with underscore
    #[token("${#")]
    DollarBraceHash,
    #[token("${!")]
    DollarBraceBang,
    #[token("${*")]
    DollarBraceStar,
    #[token("${@")]
    DollarBraceAt,
    #[token("${#*")]
    DollarBraceHashStar,
    #[token("${#@")]
    DollarBraceHashAt,
    #[token("${!*")]
    DollarBraceBangStar,
    #[token("${!@")]
    DollarBraceBangAt,

    // Arithmetic
    #[token("$((", priority = 1)]
    Arithmetic,
    #[token("$[")]
    ArithmeticBracket,
    #[token("let")]
    Let,

    // Conditionals
    #[token("-eq")]
    Eq,
    #[token("-ne")]
    Ne,
    #[token("-lt")]
    Lt,
    #[token("-le")]
    Le,
    #[token("-gt")]
    Gt,
    #[token("-ge")]
    Ge,
    #[token("-z")]
    Zero,
    #[token("-n")]
    NonZero,
    #[token("-f")]
    File,
    #[token("-d")]
    Directory,
    #[token("-e")]
    Exists,
    #[token("-r")]
    Readable,
    #[token("-w")]
    Writable,
    #[token("-x")]
    Executable,
    #[token("-s")]
    Size,
    #[token("-L")]
    Symlink,
    #[token("-h")]
    SymlinkH,
    #[token("-p")]
    PipeFile,
    #[token("-S")]
    Socket,
    #[token("-b")]
    Block,
    #[token("-c", priority = 1)]
    Character,
    #[token("-g")]
    SetGid,
    #[token("-k")]
    Sticky,
    #[token("-u")]
    SetUid,
    #[token("-O")]
    Owned,
    #[token("-G")]
    GroupOwned,
    #[token("-N")]
    Modified,
    #[token("-nt")]
    NewerThan,
    #[token("-ot")]
    OlderThan,
    #[token("-ef")]
    SameFile,

    // Strings and literals
    #[regex(r#""([^"\\]|\\.)*""#, priority = 3)]
    DoubleQuotedString,
    #[regex(r"'([^'\\]|\\[^'])*'", priority = 3)]
    SingleQuotedString,
    #[regex(r"`([^`\\]|\\.)*`", priority = 3)]
    BacktickString,
    #[regex(r"\$'([^'\\]|\\.)*'", priority = 3)]
    DollarSingleQuotedString,
    #[regex(r#"\$"([^"\\]|\\.)*""#, priority = 3)]
    DollarDoubleQuotedString,

    // Long options (must come before Identifier to avoid conflicts)
    #[regex(r"--[a-zA-Z][a-zA-Z0-9_*?.-]*=[^ \t\n\r|&;(){}]*", priority = 3)]
    LongOption,
    
    // Identifiers and words
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_*?.-]*|[.][a-zA-Z0-9_*?.-]*", priority = 2)]
    Identifier,

    #[regex(r"[0-9]+")]
    Number,
    #[regex(r"[0-9]+\.[0-9]+")]
    Float,
    #[regex(r"0x[0-9a-fA-F]+")]
    HexNumber,
    #[regex(r"0[0-7]+")]
    OctalNumber,

    // Special characters
    #[token("!")]
    Bang,
    #[token("#", priority = 1)]
    _Hash, // Unused variant, prefixed with underscore
    #[token("%")]
    Percent,
    #[token("^")]
    Caret,
    #[token("~")]
    Tilde,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/", priority = 2)]
    Slash,
    #[token("\\", priority = 1)]
    _Backslash, // Unused variant, prefixed with underscore
    #[token("?")]
    Question,
    #[token(":")]
    Colon,
    #[token("@")]
    At,
    #[token("`", priority = 2)]
    BacktickChar,
    #[token("'")]
    SingleQuote,
    #[token("\"")]
    DoubleQuote,
    #[token("\\", priority = 2)]
    Escape,
    #[token(",")]
    Comma,
    #[regex(r"\n", priority = 3)]
    Newline,
    #[token("\r")]
    CarriageReturn,
    #[token("\t")]
    Tab,
    #[token(" ")]
    Space,

    // Comments
    #[regex(r"#[^\n]*", priority = 3)]
    Comment,
}

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("Unexpected character: {ch} at {line}:{col}")]
    UnexpectedChar { ch: char, line: usize, col: usize },
    #[error("Unterminated string")]
    _UnterminatedString, // Unused variant, prefixed with underscore
    #[error("Invalid escape sequence")]
    _InvalidEscape, // Unused variant, prefixed with underscore
}

pub struct Lexer {
    tokens: Vec<(Token, usize, usize)>,
    current: usize,
    input: String,
    line_starts: Vec<usize>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut tokens = Vec::new();
        let mut lexer = Token::lexer(input);
        
        while let Some(token_result) = lexer.next() {
            let span = lexer.span();
            match token_result {
                Ok(token) => tokens.push((token, span.start, span.end)),
                Err(_) => {
                    // Skip invalid tokens
                    continue;
                }
            }
        }
        
        // Precompute starts of lines for quick offset->(line,col)
        let mut line_starts = Vec::new();
        line_starts.push(0);
        for (idx, byte) in input.as_bytes().iter().enumerate() {
            if *byte == b'\n' {
                if idx + 1 < input.len() {
                    line_starts.push(idx + 1);
                }
            }
        }

        Self { tokens, current: 0, input: input.to_string(), line_starts }
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current).map(|(token, _, _)| token)
    }

    pub fn peek_n(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.current + n).map(|(token, _, _)| token)
    }

    pub fn next(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.current).map(|(token, _, _)| token);
        self.current += 1;
        token
    }

    pub fn consume(&mut self, expected: Token) -> Result<(), LexerError> {
        if let Some(token) = self.next() {
            if std::mem::discriminant(token) == std::mem::discriminant(&expected) {
                Ok(())
            } else {
                // Get the actual character from the current token for better error reporting
                if let Some((_, start, end)) = self.tokens.get(self.current - 1) {
                    let actual_char = self.input[*start..*end].chars().next().unwrap_or('?');
                    let (line, col) = self.offset_to_line_col(*start);
                    Err(LexerError::UnexpectedChar { ch: actual_char, line, col })
                } else {
                    Err(LexerError::UnexpectedChar { ch: '?', line: 1, col: 1 })
                }
            }
        } else {
            Err(LexerError::UnexpectedChar { ch: '?', line: 1, col: 1 })
        }
    }

    pub fn is_eof(&self) -> bool {
        self.current >= self.tokens.len()
    }

    pub fn current_position(&self) -> usize {
        self.current
    }



    pub fn get_span(&self) -> Option<(usize, usize)> {
        self.tokens.get(self.current).map(|(_, start, end)| (*start, *end))
    }
    
    pub fn get_text(&self, start: usize, end: usize) -> String {
        self.input[start..end].to_string()
    }
    
    pub fn get_current_text(&self) -> Option<String> {
        self.tokens.get(self.current).map(|(_, start, end)| {
            self.input[*start..*end].to_string()
        })
    }
}

impl Lexer {

    pub fn offset_to_line_col(&self, offset: usize) -> (usize, usize) {
        if self.line_starts.is_empty() {
            return (1, offset + 1);
        }
        // Binary search for the greatest line_start <= offset
        let mut left = 0usize;
        let mut right = self.line_starts.len();
        while left < right {
            let mid = (left + right) / 2;
            match self.line_starts[mid].cmp(&offset) {
                Ordering::Greater => right = mid,
                _ => left = mid + 1,
            }
        }
        let idx = left.saturating_sub(1);
        let line_start = self.line_starts.get(idx).cloned().unwrap_or(0);
        let line = idx + 1;
        let col = offset.saturating_sub(line_start) + 1;
        (line, col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let input = "echo hello world";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next(), Some(&Token::Identifier));
        assert_eq!(lexer.next(), Some(&Token::Space));
        assert_eq!(lexer.next(), Some(&Token::Identifier));
        assert_eq!(lexer.next(), Some(&Token::Space));
        assert_eq!(lexer.next(), Some(&Token::Identifier));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_pipeline() {
        let input = "ls | grep test";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next(), Some(&Token::Identifier));
        assert_eq!(lexer.next(), Some(&Token::Space));
        assert_eq!(lexer.next(), Some(&Token::Pipe));
        assert_eq!(lexer.next(), Some(&Token::Space));
        assert_eq!(lexer.next(), Some(&Token::Identifier));
        assert_eq!(lexer.next(), Some(&Token::Space));
        assert_eq!(lexer.next(), Some(&Token::Identifier));
    }

    #[test]
    fn test_variables() {
        let input = "$HOME ${PATH}";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next(), Some(&Token::Dollar));
        assert_eq!(lexer.next(), Some(&Token::Identifier));
        assert_eq!(lexer.next(), Some(&Token::Space));
        assert_eq!(lexer.next(), Some(&Token::DollarBrace));
        assert_eq!(lexer.next(), Some(&Token::Identifier));
        assert_eq!(lexer.next(), Some(&Token::BraceClose));
    }
} 

