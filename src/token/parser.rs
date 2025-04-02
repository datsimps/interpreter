use std::fmt::Display;

use super::token::{Token, Lexer};
use super::ast::{*, self};

#[derive(Debug)]
pub enum ParseError{
    FailedIdent(Token),
    InvalidToken(Token),
    InvalidInfixOp(Token),
    InvalidPrecConversion(Token),
    InvalidPrefixOp(Token),
    InvalidStatementToken(Token),
}
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidToken(token) => write!(f, "InvalidToken: {}", token),
            ParseError::InvalidPrecConversion(token) => write!(f, "InvalidPrecConversion: {}", token),
            ParseError::InvalidPrefixOp(token) => write!(f, "InvalidPrefixOp: {}", token),
            ParseError::InvalidInfixOp(token) => write!(f, "InvalidInfixOp: {}", token),
            ParseError::FailedIdent(token) => write!(f, "Failed to read ident: token: {}", token),
            ParseError::InvalidStatementToken(token) => write!(f, "Failed to read Statement token: {}", token),
        }
    }
}
#[derive(PartialEq, PartialOrd)]
pub enum Prec {
    LOWEST,
    COMPARES,         // ==
    LESSGREATER,    // > or <
    SUM,            // +
    PRODUCT,        // *
    PREFIX,         // -X or !X
    LPAREN,
    CALL,           // myFunction(X)
}
impl Display for Prec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Prec::LOWEST => write!(f, "LOWEST"),
            Prec::COMPARES => write!(f, "COMPARES"),
            Prec::LESSGREATER => write!(f, "LESSGREATER"),
            Prec::SUM => write!(f, "SUM"),
            Prec::PRODUCT => write!(f, "PRODUCT"),
            Prec::PREFIX => write!(f, "PREFIX"),
            Prec::LPAREN => write!(f, "LPAREN"),
            Prec::CALL => write!(f, "CALL"),
        }
    }
}
pub fn token_to_prec_map(token: Token) -> Result<Prec, ParseError> {
    let prec_value = match token {
        Token::EQUAL | Token::NEQUAL => Prec::COMPARES,
        Token::LTHAN | Token::GTHAN => Prec::LESSGREATER,
        Token::PLUS | Token::SUBTRACT => Prec::SUM,
        Token::FSLASH | Token::STAR => Prec::PRODUCT,
        Token::LPAREN => Prec::LPAREN,
        toke => return Err(ParseError::InvalidPrecConversion(toke)), 
    };
    Ok(prec_value)
}
// Type alias to convet to function pointers that return expressions
//   
type PrefixParserFn = fn(&mut Parser) -> Result<Expression, ParseError>;
type InfixParserFn = fn(&mut Parser, Expression) -> Result<Expression, ParseError>;



#[allow(unused)]
pub struct Parser {
    lex: Lexer,
    cur_token: Token,
    peek_token: Token,
    errors: Vec<String>,
}
impl Default for Parser {
    fn default() -> Self {
        Parser {
            lex: Lexer::default(),
            cur_token: Token::ILLEGAL,
            peek_token: Token::ILLEGAL,
            errors: Vec::<String>::new(),
        }       
    }
}
#[allow(dead_code)]
impl Parser {
    pub fn new(l: Lexer) -> Parser {
        // lexer {input[], position, read_position, ch }
        // create the default parser then move forward twice to set values
        let mut parser = Parser::default();
        parser.lex = l;
        parser.next_token();
        parser.next_token();
        return parser;
    }
    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lex.next_token().unwrap();
    }
    fn errors(&self) -> &Vec<String> {
        return &self.errors;
    }
    fn peek_error(&mut self, token: &Token) {
        let message = format!("Expected next token to be: {}, instead found: {}",
            token.clone(), self.peek_token);
        self.errors.push(message);
    }
    pub fn parse_program(&mut self) -> ast::Program {
        // create a new program which is just an empty vector of statements
        let mut program: Program = Program::new();
        // iterate through the tokens until EOF
        while !self.cur_token_is(Token::EOF) {
            // if the statement is ok then push it to program and move forward one
            match self.parse_statement() { 
                Ok(statement) => program.statements.push(statement),
                Err(error) =>{
                    println!("error is: {}", error);
                }
            };
            self.next_token();
        }
        return program;
    }
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        // match with Let statement or ignore for now
        match &self.cur_token {
            Token::LET => return self.parse_let_statement(),
            Token::RETURN => return self.parse_return_statement(),
            _ => return self.parse_expression_statement(),
        }
    }
    fn parse_let_statement(&mut self) -> Result<Statement, ParseError> {
        // advance token passed the Token::LET
        self.next_token();
        //read ident for LetStatement.name
        let ident_name = self.read_ident()?;
        // if we dont get an assign token ./Freak_out, everything is out of order
        self.expect_peek(Token::ASSIGN)?;
        self.next_token();

        // get the remainder of the expression
        let expression = self.parse_expression(Prec::LOWEST)?;
        // move passed final semicolon
        if self.peek_token_is(&Token::SEMICOLON){
            self.next_token();
        }
        // return new LetStatement variant of STATEMENT
        Ok(
            Statement::LetStatement(
                LetStatement {
                    name: ident_name,
                    value: expression,
                }
            )
        )
    }
    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        // move passed the return token
        self.next_token();
        let expression = self.parse_expression(Prec::LOWEST)?;
        while !self.peek_token_is(&Token::SEMICOLON) {
            self.next_token();
        }
        // move passed final semicolon
        self.next_token();

        Ok(
            Statement::ReturnStatement(
                ReturnStatement {
                    ret_value: expression,
                }
            )
        )
    }
    fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let express = self.parse_expression(Prec::LOWEST)?;
        if self.peek_token_is(&Token::SEMICOLON) {
            self.next_token();
        }
        Ok(
            Statement::Expression(express)
        )
    }
    fn parse_statement_ident(&mut self) -> Result<Expression, ParseError> {
        // read ident then create the expression for Expression::Ident
        let expression = match self.read_ident() {
            Ok(ident) => Ok(Expression::Ident(ident)),
            _ => Err(ParseError::FailedIdent(self.cur_token.clone())),
        };
        return expression;
    }
    fn parse_int(&mut self) -> Result<Expression, ParseError> { 
        let expression = match &self.cur_token {
            Token::INT(num) => Ok(Expression::Literal(Literal::Int(num.clone()))),
            _ => Err(ParseError::InvalidToken(self.cur_token.clone())),
        };
        return expression;
    }
    fn parse_boolean(&mut self) -> Result<Expression, ParseError> { 
        let expression = match &self.cur_token {
            Token::TRUE => Ok(Expression::Literal(Literal::Bool(true))),
            Token::FALSE => Ok(Expression::Literal(Literal::Bool(false))),
            _ => Err(ParseError::InvalidToken(self.cur_token.clone())),
        };
        return expression;
    }
    fn parse_grouped_expression(&mut self) -> Result<Expression, ParseError> {
        self.next_token();
        let expression = self.parse_expression(Prec::LOWEST);
        self.expect_peek(Token::RPAREN)?;
        // DONT FORGET TO MOVE PASSED THE RIGHT PARENTHESIS
        if self.peek_token_is(&Token::RPAREN) {
            self.next_token();
        }
        return expression;
    }
    fn parse_block_statement(&mut self) -> Result<Statement, ParseError> {
        let mut block_statement = BlockStatement::default();
        // skip the left brace
        self.next_token();

        while !self.cur_token_is(Token::RBRACE) && !self.cur_token_is(Token::EOF) {

            let statement = self.parse_statement()?;
            block_statement.statements.push(statement);
            self.next_token();
        }
        return Ok(
            Statement::BlockStatement(
                block_statement
            )
        )
    }
    fn parse_string_literal(&mut self) -> Result<Expression, ParseError> {
        let output = self.cur_token.clone().to_string();
        return Ok(
            Expression::Literal(
                Literal::String(output)
            ))
    }
    fn parse_expression_if(&mut self) -> Result<Expression, ParseError> {
        // if (x>y) {x}
        // if (x>y) {x} else {y}
        let mut alternative = None;
        // peek token better be an LPAREN or ill freak out 
        self.expect_peek(Token::LPAREN)?;
        // skip the LPAREN
        self.next_token();
        let condition = Box::new(self.parse_expression(Prec::LOWEST)?);
       
        self.expect_peek(Token::RPAREN)?;
        self.expect_peek(Token::LBRACE)?;

        let consequence = if let Statement::BlockStatement(block_statement) = self.parse_block_statement()? {
            block_statement
        } else {
            return Err(ParseError::InvalidStatementToken(self.cur_token.clone()))
        };

        if self.peek_token_is(&Token::ELSE) {
            //skip right brace
            self.next_token();
            self.expect_peek(Token::LBRACE)?;
            //cur_token is no left brace
            // if there is a passable BlockStatement create the BlockStatement assign it to alternative
            alternative = if let Statement::BlockStatement(block_statement) = self.parse_block_statement()? {
                Some(block_statement)
            } else {
                return Err(ParseError::InvalidStatementToken(self.cur_token.clone()))
            }
        }
        return Ok( 
                Expression::IfExpression(
                    IfExpression {
                        condition, 
                        consequence,
                        alternative,
                    })
            )

    }
    fn parse_function_parameters(&mut self) -> Result<Parameters, ParseError> {
        // input: FN(), FN(x), FN(x,y)
        let mut variables = Vec::<Expression>::new();
        if self.peek_token_is(&Token::RPAREN) {
            self.next_token();
            return Ok(Parameters { variables } );
        }
        // skip the LPAREN
        self.next_token();

        // stop if RPAREN, checks for single variable 
        while !self.cur_token_is(Token::RPAREN) && !self.cur_token_is(Token::EOF) {
            // if multiple variables, check for comma before parsing 
            if self.cur_token_is(Token::COMMA) {
                self.next_token();
            }
            let idents = self.parse_statement_ident()?;
            variables.push(idents);
            self.next_token();
        }
        return Ok(
                Parameters{
                    variables,
                }
        );
            
    }
    fn parse_expression_function(&mut self) -> Result<Expression, ParseError> {
        //input FN (...) {...}
        //check for LPAREN then skip it
        self.expect_peek(Token::LPAREN)?;
        //IF there are parameters assign them to parameters else NONE
        let parameters = self.parse_function_parameters()?;   
        // need BlockStatement for FN call
        self.expect_peek(Token::LBRACE)?;
        let body = if let Statement::BlockStatement(block_statement) = self.parse_block_statement()? {
            block_statement
        } else {
            return Err(ParseError::InvalidStatementToken(self.cur_token.clone()));
        };

        return Ok(
            Expression::FunctionLiteral(
                FunctionLiteral {
                    parameters,
                    body,
                }
            )
        )
    }
    fn parse_expression_prefix(&mut self) -> Result<Expression, ParseError> {
        // map the token to a prefix operator
        let prefix_op = match self.cur_token.clone() {
            Token::BANG => PrefixOp::BANG,
            Token::SUBTRACT => PrefixOp::NEGATIVE,
            _ => return Err(ParseError::InvalidToken(self.cur_token.clone())),
        };
        // move token forward one from the prefix operator
        self.next_token();

        // parse the rest of the expression and create a PrefixExpression
        let expression = match self.parse_expression(Prec::PREFIX) {
            Ok(expression) => Ok(
                Expression::PrefixExpression(
                    PrefixExpression {
                        prefix_op,
                        expression: Box::new(expression)
                    }
                )
            ),
            Err(error) => Err(error),
        };
        return expression;
    }
    fn parse_expression_infix(&mut self, express: Expression) -> Result<Expression, ParseError> {

        // map the token to a Infix operator
        let infix_op = match self.cur_token {
            Token::PLUS => InfixOp::PLUS,
            Token::SUBTRACT => InfixOp::SUBTRACT,
            Token::STAR => InfixOp::MULTIPLY,
            Token::FSLASH => InfixOp::DIVIDE,
            Token::LTHAN => InfixOp::LESSTHAN,
            Token::GTHAN => InfixOp::GREATERTHAN,
            Token::EQUAL => InfixOp::COMPARE,
            Token::NEQUAL => InfixOp::NEQUALS,
            _ => return Err(ParseError::InvalidToken(self.cur_token.clone())),
        };

        let prec = self.cur_prec_is();

        // move token forward one from the infix operator
        self.next_token();

        // parse expression and if the expression (expression) is okay then
        // create an InfixExpression
        let expression = match self.parse_expression(prec) {
            Ok(expression) => Ok(
                Expression::InfixExpression(
                    InfixExpression {
                        left_expression: Box::new(express),
                        infix_op,
                        right_expression: Box::new(expression)
                    }
                )
            ),
            Err(_) => Err(ParseError::InvalidInfixOp(self.cur_token.clone())),
        };
        return expression;
    }

    fn parse_arguments(&mut self) -> Result<Arguments, ParseError> {
        // input (); , (x); , (x + y); , (x, x + y, x * y);
        let mut variables = Vec::<Expression>::new();
        //check for zero args
        if self.peek_token_is(&Token::RPAREN) {
            return Ok(Arguments {variables} );
        }
        //skip LPAREN
        self.next_token();

        while !self.cur_token_is(Token::RPAREN) & !self.cur_token_is(Token::SEMICOLON){
            // x, x + y, x * y);
            // if comma, skip comma
            if self.cur_token_is(Token::COMMA) {
                self.next_token();
            }
            let arg = self.parse_expression(Prec::LOWEST)?;
            variables.push(arg);
            self.next_token();
        }
        return Ok(
                Arguments { variables }
               );
    }
    fn parse_expression_call(&mut self, expression: Expression) -> Result<Expression, ParseError> {
        // input: FN(), FN(x), FN(x,y)
        let arguments = self.parse_arguments()?;
       return Ok(
            Expression::CallExpression(
                CallExpression {
                    function: Box::new(expression), 
                    arguments,
                }
            )
        );
    }
    fn prefix_parser(&self) -> Option<PrefixParserFn> {
        // match the token to the functions we need to use to create expressions
        Some(
            match self.cur_token {
                Token::IDENT(_) => Parser::parse_statement_ident,
                Token::INT(_) => Parser::parse_int,
                Token::BANG | Token::SUBTRACT => Parser::parse_expression_prefix, 
                Token::TRUE | Token::FALSE => Parser::parse_boolean,
                Token::LPAREN => Parser::parse_grouped_expression,
                Token::IF => Parser::parse_expression_if,
                Token::FUNCTION => Parser::parse_expression_function,
                Token::STRING(_) => Parser::parse_string_literal,
                _ => return None,
            }
        )
    }
    
    fn infix_parser(&mut self) -> Option<InfixParserFn> {
        // match the token to the functions we need to use to create expressions
        // we match on the peek token: 
        Some(
            match self.peek_token {
                Token::PLUS 
                | Token::SUBTRACT
                | Token::STAR
                | Token::FSLASH
                | Token::GTHAN
                | Token::LTHAN
                | Token::EQUAL
                | Token::NEQUAL => Parser::parse_expression_infix,
                Token::LPAREN => Parser::parse_expression_call,
                _ => return None,
            }
        )
    }
    
    fn parse_expression(&mut self, prec: Prec) -> Result<Expression, ParseError> {
        //prefix examples:      infix examples:       we get left_exp(prefix or values)
        //  input = "foobar"    input = "5 + 5"       so "literal(INT(5))" check NEXT token "t.PLUS"
        //  input = "!true"     input = "5 * 5"       map that to an InfixOp (still peek_token) 
        //  input = "-5"        input = "5 == 5"      get rest of expression
        //  get the prefix if any, or return the literals as an expression 
        let prefix = self.prefix_parser().ok_or_else(|| ParseError::InvalidPrefixOp(self.cur_token.clone()))?;

        let mut left_exp = prefix(self)?;

        // if next token is a semicolon just skip and return left_exp i.e. "++i;" there is no Infix
        // else: 1 + 2; 1 is prefix, we check peek: {+}, so parse the infix and right exp
        while !self.peek_token_is(&Token::SEMICOLON) && prec < self.peek_prec_is() {
            let infix = self.infix_parser().ok_or_else(|| ParseError::InvalidInfixOp(self.cur_token.clone()))?;
            self.next_token();
            left_exp = infix(self, left_exp)?;
        };

        // return left_exp or left_exp = infixed(left_expression)
        return Ok(left_exp);
    }

    fn read_ident(&mut self) -> Result<String, ParseError> {
        // match token 
        match &self.cur_token {
            Token::IDENT(string) => Ok(string.clone()),
            _ => Err(ParseError::FailedIdent(self.cur_token.clone()))
        }
    }
    fn cur_token_is(&self, t: Token) -> bool {
        if self.cur_token == t {
            return true;
        } else {
            return false;
        }
    }
    fn cur_prec_is(&self) -> Prec {
        if let Ok(prec) = token_to_prec_map(self.cur_token.clone()) {
            return prec;
        } else {
            return Prec::LOWEST;
        }
    }
    fn peek_token_is(&self, t: &Token) -> bool {
        if &self.peek_token == t {
            return true;
        } else {
            return false;
        }
    }
    fn peek_prec_is(&self) -> Prec {
        if let Ok(prec) = token_to_prec_map(self.peek_token.clone()) {
            return prec;
        } else {
            return Prec::LOWEST;
        }
    }   
    fn expect_peek(&mut self, t: Token) -> Result<(), ParseError> {
        if self.peek_token_is(&t) {
            self.next_token();
            return Ok(());
        } else {
            self.peek_error(&t);
            return Err(ParseError::InvalidToken(self.peek_token.clone()));
        }
    }
    pub fn check_parsing_errors(&self) -> () {
        let errors = &self.errors;
        if errors.len() == 0 {
            return;
        }
        println!("Make it so");
        let face = r#"
                    ___
       ___....-----'---'-----....___
 =========================================
        ___'---..._______...---'___
       (___)      _|_|_|_      (___)
         \\____.-'_.---._'-.____//
          cccc'.___'---'____.'cccc
                   ccccc
        "#;
        println!("{}", face);
        for message in errors {
            println!("{message}");
        }
        return;
    }
}


#[cfg(test)]
mod test{
    use anyhow::Result;
    use crate::token::ast::{LetStatement,Statement,Program,Expression};

    use super::Lexer;
    use super::Parser;

    #[test]
    fn test_let_statements() -> Result<(), String> {
        let input = "
        let x = 5;
        let y = 10;
        let foobar = 838383;
        ".to_string();
   
        // create a tokens of the input
        let l = Lexer::new(input);
        // create a parser to move through the lines
        let mut p = Parser::new(l);
        // create the program to hold the lines
        let program = p.parse_program();
        check_parsing_errors(p);

        // parse program and see if we got three entries
        if program.statements.len() != 3 {
            return Err(format!("Program statements does not contain three statements, got: {}", program.statements.len()));
        }
       
        // these should be the names of the statements
        let names = vec![
            "x",
            "y",
            "foobar",
        ];
        let expressions = vec![
            "5",
            "10",
            "838383",
        ];
        
        // test that the each statement name equals the names entries
        for i in 0..names.len() {
            let statement = program.statements[i as usize].clone();
            let name = statement.get_statement_name();
            let express = statement.get_expression();
            println!("Expected name: {}, got: {}", names[i], name);
            assert_eq!(name, names[i].to_string());
            println!("Expected expression: {}, got: {}", expressions[i], express);
            assert_eq!(express, expressions[i]);
 

        }
        Ok(())
    }
    fn check_parsing_errors(p: Parser) -> () {
        let errors = &p.errors;
        if errors.len() == 0 {
            return;
        }
        println!("Parser has {} errors", errors.len());

        for message in errors {
            println!("{message}");
        }
        return;
    }
    #[test]
    fn test_return_statements() -> Result<(), String> {
        let input = "
        return 5;
        return 10;
        return 993322;
        ".to_string();
   
        // create a tokens of the input
        let l = Lexer::new(input);
        // create a parser to move through the lines
        let mut p = Parser::new(l);
        // create the program to hold the lines
        let program = p.parse_program();
        check_parsing_errors(p);

        // parse program and see if we got three entries
        if program.statements.len() != 3 {
            return Err(format!("Program statements does not contain three statements, got: {}", program.statements.len()));
        }
       
        // these should be the names of the statements
        let values = vec![
            "5",
            "10",
            "993322",
        ];
        let expressions = vec![
            "5",
            "10",
            "993322",
        ];
        
        // test that the each statement name equals the names entries
        for i in 0..values.len() {
            let statement = program.statements[i as usize].clone();
            let name = statement.get_statement_name();
            let express = statement.get_expression();
            println!("Expected name: {}, got: {}", values[i], name);
            assert_eq!(name, values[i].to_string());
            println!("Expected expression: {}, got: {}", expressions[i], express);
            assert_eq!(express, expressions[i]);
        }
        Ok(())
    }

    #[test]
    fn test_string() -> Result<(), String>{
        let _input = "let myVar = anotherVar;";
        let statement =  Statement::LetStatement(
                                LetStatement {
                                    name: "myVar".to_string(),
                                    value: Expression::Ident("anotherVar".to_string()),
                                }
                            );
        let program = Program{
            statements: vec![statement]
        };

        for i in 0..program.statements.len() {
            println!("{}", program.statements[i]);
        };
        
        assert_eq!(program.statements[0].get_statement_name(), "myVar");
        assert_eq!(program.statements[0].get_expression(), "anotherVar");
        Ok(())
    }
    #[test]
    fn test_prefix() -> Result<(), String> {
        let input = "
            !5;
            -15;
            true;
            false;
        ".to_string();

        let values = vec![
        "(! 5)",
        "(- 15)",
        "true",
        "false"
        ];
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parsing_errors(p);
        
        if program.statements.len() != 4 {
            return Err(format!("Program statements does not contain four statements, got: {}", program.statements.len()));
        }

        for i in 0..values.len() {
            let statement = program.statements[i as usize].clone();
            let name = statement.get_expression();
            println!("Expected name: {}, got: {}", values[i], name);
            assert_eq!(name, values[i].to_string());
        }
        Ok(())
    }
    #[test]
    fn parse_infix_expression() -> Result<(), String> {
        let input = "
            5 + 5;
            5 - 5;
            5 * 5;
            5 / 5;
            5 > 5;
            5 < 5;
            5 == 5;
            5 != 5;
            !true;
        ".to_string();
        
        let values = vec![
          "(5 + 5)",
          "(5 - 5)",
          "(5 * 5)",
          "(5 / 5)",
          "(5 > 5)",
          "(5 < 5)",
          "(5 == 5)",
          "(5 != 5)",
          "(! true)"
        ];
        
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parsing_errors(p);
        
        if program.statements.len() != 9 {
            return Err(format!("Program statements does not contain nine statements, got: {}", program.statements.len()));
        }

        for i in 0..values.len() {
            let statement = program.statements[i as usize].clone();
            let name = statement.get_expression();
            println!("Expected name: {}, got: {}", values[i], name);
            assert_eq!(name, values[i].to_string());
        }
        Ok(())
    }

    #[test]
    fn test_precedence_parsin() -> Result<(), String> {
        let input = "
            -a * b;
            !-a;
            a + b + c;
            a + b - c;
            a * b * c;
            a * b / c;
            a + b / c;
            3 + 4 * 5 == 3 * 1 + 4 * 5;
            true == true;
            3 < 5 == true;
            1 + (2 + 3) * 4;
            -(5 + 5);
        ".to_string();
        
        let values = vec![
          "((- a) * b)",
          "(! (- a))",
          "((a + b) + c)",
          "((a + b) - c)",
          "((a * b) * c)",
          "((a * b) / c)",
          "(a + (b / c))",
          "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
          "(true == true)",
          "((3 < 5) == true)",
          "(1 + ((2 + 3) * 4))",
          "(- (5 + 5))",
        ];
        
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parsing_errors(p);
        
        if program.statements.len() != 12 {
            return Err(format!("Program statements does not contain 12 statements, got: {}", program.statements.len()));
        }

        for i in 0..values.len() {
            let statement = program.statements[i as usize].clone();
            let name = statement.get_expression();
            assert_eq!(values[i].to_string(), name);
        }
        Ok(())
    }
    
    #[test]
    fn test_if_expressions() -> Result<(), String> {
        let input = "if (x < y) { x }".to_string();
        
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parsing_errors(p);
        
        if program.statements.len() != 1 {
            return Err(format!("Program statements does not contain 1 statement, got: {}", program.statements.len()));
        }

        let values = vec![
            "IF (x < y) { x }",
        ];
        for i in 0..values.len() {
            let statement = program.statements[i as usize].clone();
            let name = statement.get_expression();
            println!("Statement: {}", &name);
            assert_eq!(values[i].to_string(), name);
        };
        Ok(())
    }
    #[test]
    fn test_if_else_expressions() -> Result<(), String> {
        let input = "
        if (a < b) { a } else { b };
        ".to_string();
        
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parsing_errors(p);
        
        if program.statements.len() != 1 {
            return Err(format!("Program statements does not contain 1 statement, got: {}", program.statements.len()));
        }

        let values = vec![
            "IF (a < b) { a } ELSE { b }"
        ];
        for i in 0..values.len() {
            let statement = program.statements[i as usize].clone();
            let name = statement.get_expression();

            assert_eq!(values[i].to_string(), name);
        };
        Ok(())
    }
    #[test]
    fn test_function_literal() -> Result<(), String> {
        let input = "
        fn(x,y) {x + y;}
        ".to_string();
        
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parsing_errors(p);
        
        if program.statements.len() != 1 {
            return Err(format!("Program statements does not contain 1 statement, got: {}", program.statements.len()));
        }

        let values = vec![
            "FN (x,y) { (x + y) }"
        ];
        for i in 0..values.len() {
            let statement = program.statements[i as usize].clone();
            let name = statement.get_expression();

            println!("{}", statement);
            assert_eq!(values[i].to_string(), name);
        };

        Ok(())
    } 
    #[test]
    fn test_function_call() -> Result<(), String> {
        let input = "
        add(1,2 * 3,4 + 5);
        add(1, sub(2+4));
        ".to_string();
        
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parsing_errors(p);
        
        if program.statements.len() != 2 {
            return Err(format!("Program statements does not contain 2 statement, got: {}", program.statements.len()));
        }

        let values = vec![
            "FN (add) {1,(2 * 3),(4 + 5)}",
            "FN (add) {1,FN (sub) {(2 + 4)}}",
        ];
        for i in 0..values.len() {
            let statement = program.statements[i as usize].clone();
            let name = statement.get_expression();

            assert_eq!(values[i].to_string(), name);
        };

        Ok(())
    }
    #[test]
    fn test_string_literal_expression() -> Result<(), String> {
        let input = r#""hello world""#.to_string();
        
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parsing_errors(p);
        
        if program.statements.len() != 2 {
            for i in &program.statements {
                println!("{}", i);
            }
            return Err(format!("Program statements does not contain a single statement, got: {}", program.statements.len()));
        }

        let values = vec![
            "hello world",
        ];
        for i in 0..values.len() {
            let statement = program.statements[i as usize].clone();
            let name = statement.get_expression();

            assert_eq!(values[i].to_string(), name);
        };

        Ok(())
    }

}
