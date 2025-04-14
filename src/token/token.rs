use anyhow::Result;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    //Specials
    ILLEGAL,
    EOF,

    // Identifiers + Literals
    IDENT(String),
    INT(String),
    STRING(String),

    // Operators
    ASSIGN,
    PLUS,
    SUBTRACT,
    FSLASH,
    BANG,
    STAR,
    LTHAN,
    GTHAN,
    LEQUAL,
    GEQUAL,
    EQUAL,
    NEQUAL,

    // Delimeters
    COMMA,
    SEMICOLON,

    LPAREN,
    RPAREN,
    LBRACE,
    RBRACE,

    // Keywords
    FUNCTION,
    LET,
    IF,
    ELSE,
    TRUE,
    FALSE,
    RETURN,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Token::ILLEGAL => write!(f, "ILLEGAL"),
            Token::EOF => write!(f, "EOF"),
            Token::IDENT(string) => write!(f, "IDENT({})", string),
            Token::INT(string) => write!(f, "{}", string),
            Token::STRING(string) => write!(f, "{}", string),
            Token::ASSIGN => write!(f, "ASSIGN"),
            Token::BANG => write!(f, "BANG"),
            Token::PLUS => write!(f, "PLUS"),
            Token::SUBTRACT => write!(f, "SUBTRACT"),
            Token::FSLASH => write!(f, "FSLASH"),
            Token::STAR => write!(f, "STAR"),
            Token::LTHAN => write!(f, "LTHAN"),
            Token::GTHAN => write!(f, "GTHAN"),
            Token::LEQUAL => write!(f, "LEQUAL"),
            Token::GEQUAL => write!(f, "GEQUAL"),
            Token::EQUAL => write!(f, "EQUAL"),
            Token::NEQUAL => write!(f, "NEQUAL"),
            Token::COMMA => write!(f, "COMMA"),
            Token::SEMICOLON => write!(f, "SEMICOLON"),
            Token::LPAREN => write!(f, "LPAREN"),
            Token::RPAREN => write!(f, "RPAREN"),
            Token::LBRACE => write!(f, "LBRACE"),
            Token::RBRACE => write!(f, "RBRACE"),
            Token::FUNCTION => write!(f, "FUNCTION"),
            Token::LET => write!(f, "LET"),
            Token::IF => write!(f, "IF"),
            Token::ELSE => write!(f, "ELSE"),
            Token::TRUE => write!(f, "TRUE"),
            Token::FALSE => write!(f, "FALSE"),
            Token::RETURN => write!(f, "RETURN"),
        }
    }
}
#[allow(unused)]
impl Token {
    fn to_string(&self) -> String {
        let output: String = format!("{}", self).to_string();
        return output;
    }
}

pub struct Lexer {
   pub input: Vec<u8>,
   pub position: usize,
   pub read_position: usize,
   pub ch: u8,
}
impl Default for Lexer {
    fn default() -> Self {
        Self {
        input: Vec::<u8>::new(),
        position: 0,
        read_position: 0,
        ch: 0
        }
    }
}
impl Lexer {
    // Set default variables for lexer
    pub fn new(input: String) -> Lexer {
        let mut lex = Lexer::default();
        lex.input = input.into_bytes();
        lex.read_char();
        return lex;
    }

    // Take a look at the input then return the next char
    fn read_char(&mut self) -> u8 {
        // if we reached the EOF char returns 0 (EOF)
        //  else set ch to the next positon
        if self.read_position >= self.input.len() {
            self.ch = 0;
        } else {
            self.ch = self.input[self.read_position];
        }
        self.position = self.read_position;
        self.read_position += 1;
        return self.ch
    }

    pub fn next_token(&mut self)-> Result<Token> {
        // " let x = 4;"
        // skip whitespace = "let x = 4;"
        // match for let = "{Token::LET} x = 4;"
        // read_char to move up = Token::LET + " x = 4;"
        // return Token::LET;
        self.skip_whitespace();
        let tok = match self.ch {
            b'=' => {
                let next_char = self.peek_char();
                if next_char == b'=' {
                    self.read_char();
                    Token::EQUAL
                } else {
                    Token::ASSIGN
                }
            },      // =, ==
            b';' => Token::SEMICOLON,
            b'(' => Token::LPAREN,
            b')' => Token::RPAREN,
            b'{' => Token::LBRACE,
            b'}' => Token::RBRACE,
            b',' => Token::COMMA,
            b'+' => Token::PLUS,
            b'-' => Token::SUBTRACT,
            b'/' => Token::FSLASH,
            b'*' => Token::STAR,
            b'!' => {
                let next_char = self.peek_char();
                if next_char == b'=' {
                    self.read_char();
                    Token::NEQUAL
                } else {
                    Token::BANG
                }
            },        // !=, !
            b'<' => {
                let next_char = self.peek_char();
                if next_char == b'=' {
                    self.read_char();
                    Token::LEQUAL
                } else {
                    Token::LTHAN
                }
            },       // >=, <=
            b'>' => {
                let next_char = self.peek_char();
                if next_char == b'=' {
                    self.read_char();
                    Token::GEQUAL
                } else {
                    Token::GTHAN
                }
            },
            b'"' => {
                let string = self.read_string();
                return Ok( Token::STRING(string) )
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let ident = self.read_ident();
                return Ok(match ident.as_str() {
                    "fn" => Token::FUNCTION,
                    "let" =>  Token::LET,
                    "if" => Token::IF,
                    "else" => Token::ELSE,
                    "true" => Token::TRUE,
                    "false" => Token::FALSE,
                    "return" => Token::RETURN,
                    _ => Token::IDENT(ident),
                });
            },
            b'0'..=b'9' => {
                return Ok(Token::INT(self.read_number()))
            },
            0 => Token::EOF,
            i => {
                if i.is_ascii() {
                    let shitty = vec![i];
                    let output = std::str::from_utf8(&shitty)?;
                    unreachable!("Unreachable option for TOKEN: {output}")
                } else {
                    unreachable!("Unreachable option for TOKEN {i}")
                }
            },
        };
        self.read_char();
        return Ok(tok)
    }

    fn peek_char(&mut self) -> u8 {
        if self.position > self.read_position {
            return 0
        } else {
            return self.input[self.read_position]
        }
    }

    fn is_letter(ch: u8) -> bool {
        if ch >= b'a' && ch <= b'z' || ch >= b'A' && ch <= b'Z' || ch == b'_' {
            return true
        } else {
            return false 
        }
    }

    fn read_ident(&mut self) -> String {
       let position = self.position;
       while Lexer::is_letter(self.ch) {
           self.read_char();
       }
       let output = std::str::from_utf8(&self.input[position..self.position]).unwrap().to_string();
       return output
    }
    fn read_string(&mut self) -> String {
        // input = "hello world"
        //  skip the '"'
        self.read_char();
        // input = hello world"
        let position = self.position;
        while Lexer::is_still_string(self.ch) {
            println!("{}", &self.ch);
            self.read_char();
        }
        let output = std::str::from_utf8(&self.input[position..self.position]).unwrap().to_string();
        self.skip_whitespace();
        self.read_char();
        return output
    }
    fn is_number(ch: u8) -> bool {
        if ch >= b'0' && ch <= b'9' {
            return true 
        } else {
            return false
        }
    }
    fn is_still_string(ch: u8) -> bool {
        if ch == b'"' || ch == 0 {
            return false
        } else {
            return true 
        }
    }
    fn read_number(&mut self) -> String {
        let position = self.position;
        while Lexer::is_number(self.ch) {
            self.read_char();
        }
        let output = std::str::from_utf8(&self.input[position..self.position]).unwrap().to_string();
        return output
    }
    
    fn skip_whitespace(&mut self) {
        while self.ch == b' ' || self.ch == b'\t' || self.ch == b'\n' || self.ch == b'\r' {
            self.read_char();
        }
    }
}

#[cfg(test)]
mod test{
    use anyhow::Result;
    use super::{Lexer, Token};

    #[test]
    pub fn test_next_token() -> Result<()> {
        let input = "=+(){},;".to_string();
        let options = vec![
        Token::ASSIGN,
        Token::PLUS,
        Token::LPAREN,
        Token::RPAREN,
        Token::LBRACE,
        Token::RBRACE,
        Token::COMMA,
        Token::SEMICOLON,
        Token::EOF,
        ];

        let mut lex = Lexer::new(input);
        for entry in options {
            let token = lex.next_token()?;
            println!("Expected: {entry}, got: {token}");
            assert_eq!(entry, token);
        }
        Ok(())
    }
    
    #[test]
    pub fn next_token_int_and_function() -> Result<()> {
        let input = "let five = 5;
        let ten = 10;

        let add = fn(x, y) {
            x + y;
        };
        let result = add(five, ten);
        ".to_string();
        let options = vec![
        Token::LET,
        Token::IDENT("five".to_string()),
        Token::ASSIGN,
        Token::INT("5".to_string()),
        Token::SEMICOLON,
        Token::LET,
        Token::IDENT("ten".to_string()),
        Token::ASSIGN,
        Token::INT("10".to_string()),
        Token::SEMICOLON,
        Token::LET,
        Token::IDENT("add".to_string()),
        Token::ASSIGN,
        Token::FUNCTION,
        Token::LPAREN,
        Token::IDENT("x".to_string()),
        Token::COMMA,
        Token::IDENT("y".to_string()),
        Token::RPAREN,
        Token::LBRACE,
        Token::IDENT("x".to_string()),
        Token::PLUS,
        Token::IDENT("y".to_string()),
        Token::SEMICOLON,
        Token::RBRACE,
        Token::SEMICOLON,
        Token::LET,
        Token::IDENT("result".to_string()),
        Token::ASSIGN,
        Token::IDENT("add".to_string()),
        Token::LPAREN,
        Token::IDENT("five".to_string()),
        Token::COMMA,
        Token::IDENT("ten".to_string()),
        Token::RPAREN,
        Token::SEMICOLON,
        Token::EOF,
        ];
        
        let mut lex = Lexer::new(input);
        for entry in options {
            let token = lex.next_token()?;
            println!("Expected: {entry}, got: {token}");
            assert_eq!(entry, token);
        }
        Ok(())
    }
    #[test]
    pub fn next_token_added_operators() -> Result<()> {
        let input = "let five = 5;
        let ten = 10;

        let add = fn(x, y) {
            x + y;
        };
        let result = add(five, ten);
        !-/*5;
        5 < 10 > 5; 
        ".to_string();
        let options = vec![
        Token::LET,
        Token::IDENT("five".to_string()),
        Token::ASSIGN,
        Token::INT("5".to_string()),
        Token::SEMICOLON,
        Token::LET,
        Token::IDENT("ten".to_string()),
        Token::ASSIGN,
        Token::INT("10".to_string()),
        Token::SEMICOLON,
        Token::LET,
        Token::IDENT("add".to_string()),
        Token::ASSIGN,
        Token::FUNCTION,
        Token::LPAREN,
        Token::IDENT("x".to_string()),
        Token::COMMA,
        Token::IDENT("y".to_string()),
        Token::RPAREN,
        Token::LBRACE,
        Token::IDENT("x".to_string()),
        Token::PLUS,
        Token::IDENT("y".to_string()),
        Token::SEMICOLON,
        Token::RBRACE,
        Token::SEMICOLON,
        Token::LET,
        Token::IDENT("result".to_string()),
        Token::ASSIGN,
        Token::IDENT("add".to_string()),
        Token::LPAREN,
        Token::IDENT("five".to_string()),
        Token::COMMA,
        Token::IDENT("ten".to_string()),
        Token::RPAREN,
        Token::SEMICOLON,
        Token::BANG,
        Token::SUBTRACT,
        Token::FSLASH,
        Token::STAR,
        Token::INT("5".to_string()),
        Token::SEMICOLON,
        Token::INT("5".to_string()),
        Token::LTHAN,
        Token::INT("10".to_string()),
        Token::GTHAN,
        Token::INT("5".to_string()),
        Token::SEMICOLON,
        Token::EOF,
        ];
        
        let mut lex = Lexer::new(input);
        for entry in options {
            let token = lex.next_token()?;
            println!("Expected: {entry}, got: {token}");
            assert_eq!(entry, token);
        }
        Ok(())
    }
    #[test]
    pub fn next_token_keywords() -> Result<()> {
        let input = "let five = -5;
        let ten = 10;

        let add = fn(x, y) {
            x + y;
        };
        let result = add(five, ten);
        !-/*5;
        5 < 10 > 5;
        if (5 < 10) {
          return true;
        } else {
          return false;
        }
        10 == 10;
        10 != 9;
        ".to_string();
        let options = vec![
        Token::LET,
        Token::IDENT("five".to_string()),
        Token::ASSIGN,
        Token::SUBTRACT,
        Token::INT("5".to_string()),
        Token::SEMICOLON,
        Token::LET,
        Token::IDENT("ten".to_string()),
        Token::ASSIGN,
        Token::INT("10".to_string()),
        Token::SEMICOLON,
        Token::LET,
        Token::IDENT("add".to_string()),
        Token::ASSIGN,
        Token::FUNCTION,
        Token::LPAREN,
        Token::IDENT("x".to_string()),
        Token::COMMA,
        Token::IDENT("y".to_string()),
        Token::RPAREN,
        Token::LBRACE,
        Token::IDENT("x".to_string()),
        Token::PLUS,
        Token::IDENT("y".to_string()),
        Token::SEMICOLON,
        Token::RBRACE,
        Token::SEMICOLON,
        Token::LET,
        Token::IDENT("result".to_string()),
        Token::ASSIGN,
        Token::IDENT("add".to_string()),
        Token::LPAREN,
        Token::IDENT("five".to_string()),
        Token::COMMA,
        Token::IDENT("ten".to_string()),
        Token::RPAREN,
        Token::SEMICOLON,
        Token::BANG,
        Token::SUBTRACT,
        Token::FSLASH,
        Token::STAR,
        Token::INT("5".to_string()),
        Token::SEMICOLON,
        Token::INT("5".to_string()),
        Token::LTHAN,
        Token::INT("10".to_string()),
        Token::GTHAN,
        Token::INT("5".to_string()),
        Token::SEMICOLON,
        Token::IF,
        Token::LPAREN,
        Token::INT("5".to_string()),
        Token::LTHAN,
        Token::INT("10".to_string()),
        Token::RPAREN,
        Token::LBRACE,
        Token::RETURN,
        Token::TRUE,
        Token::SEMICOLON,
        Token::RBRACE,
        Token::ELSE,
        Token::LBRACE,
        Token::RETURN,
        Token::FALSE,
        Token::SEMICOLON,
        Token::RBRACE,
        Token::INT("10".to_string()),
        Token::EQUAL,
        Token::INT("10".to_string()),
        Token::SEMICOLON,
        Token::INT("10".to_string()),
        Token::NEQUAL,
        Token::INT("9".to_string()),
        Token::SEMICOLON,
        Token::EOF,
        ];
        
        let mut lex = Lexer::new(input);
        for entry in options {
            let token = lex.next_token()?;
            println!("Expected: {entry}, got: {token}");
            assert_eq!(entry, token);
        }
        Ok(())
    }
    
    #[test]
    pub fn test_token_string() -> Result<()> {
        let input = vec![r#""foobar""#, r#""foo bar""#];

        let options = vec![
                Token::STRING("foobar".to_string()),
                Token::STRING("foo bar".to_string()),
        ];
        
        for entry in 0..options.len() {
            let mut lex = Lexer::new(input[entry].to_string());
            let token = lex.next_token()?;
            println!("Expected: {entry}, got: {token}");
            assert_eq!(options[entry], token);
        }
        Ok(())
    }
}
