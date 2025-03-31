use anyhow::Result;

use crate::token::object::ObjectType;

use super::token::Lexer;
use super::parser::Parser;
use super::evaluator::Evalulator;
use super::ast;

pub fn start() -> Result<()> {
    println!("Input the commands to be translated: ");

    let mut evalulator = Evalulator::new();
    std::io::stdin().lines().for_each(|input| {
        if let Ok(input) = input {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);
            let program = p.parse_program();
            p.check_parsing_errors();

            let statement = program.statements[0].clone();
            let name = statement.get_statement_name();
            let express = statement.get_expression();
            println!("Name: {}", name);
            println!("Expression: {}", express);
            let evalulated = evalulator.eval(ast::Node::Program(program)).unwrap();

            println!("{}", evalulated.inspect());
        }
    });

    Ok(())
}
