use std::fmt::Display;
use crate::token::object::{Object, Integer, Boolean, Return, Environment};
use crate::token::ast::{Node,Expression,Literal, Program, PrefixOp, Arguments};
use super::ast::{Statement, InfixOp, BlockStatement, IfExpression, ReturnStatement, LetStatement, FunctionLiteral, CallExpression};
use super::object::{ObjectType, Function};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum EvalError{
    FailedEval(String),
    FailedExpression(Expression),
    Mismatched(Object, InfixOp, Object),
    UnknownPrefix(Object, PrefixOp),
    UnknownInfix(Object, InfixOp, Object),
    UnknownIdent(String),
    FailedObject(Object),
}
impl Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::FailedEval(string) => write!(f, "FailedEval: {}", string),
            EvalError::Mismatched(obj1, infix, obj2) => write!(f, "Unknown operation: {} {} {}", obj1.obj_type(), infix, obj2.obj_type()),
            EvalError::UnknownPrefix(obj1, prefix) => write!(f, "Unknown Operator: {}{}", prefix, obj1.obj_type()),
            EvalError::UnknownInfix(obj1, infix, obj2) => write!(f, "Unknown operation: {} {} {}", obj1.obj_type(), infix, obj2.obj_type()),
            EvalError::UnknownIdent(string) => write!(f, "Unknown Identifier: {}", string),
            EvalError::FailedObject(obj1) => write!(f, "Use of an unsupported object: {}", obj1.obj_type()),
            EvalError::FailedExpression(express) => write!(f, "Failed Evaluation of expression: {}", express),
        }
    }
}



pub struct Evalulator {
    // environment used to keep track up variable bindings
    // using RefCell to allow interior mutatability
    pub environment: RefCell<Environment>,
}

impl Evalulator {
    pub fn new() -> Self {
        Evalulator {
            environment: RefCell::new(Environment::new()),
        }
    }
    pub fn print(&self) -> () {
        self.environment.borrow_mut().print();
    }
    pub fn new_error(&self, input: String) -> Result<Object, EvalError> {
       return Ok(Object::Error(input)); 
    }

    pub fn eval(&mut self, node: Node) -> Result<Object, EvalError> {
        // three Node types
        match node {
            Node::Program(program) => self.eval_program(program),
            Node::Statement(statement) => match statement {
                  Statement::LetStatement(let_) => self.eval_let_statement(let_),
                  Statement::BlockStatement(block) => self.eval_block_statement(block),
                  Statement::ReturnStatement(ret) => self.eval_ret_statement(ret),
                  Statement::Expression(express) => self.eval(Node::Expression(express)),
            },
            Node::Expression(express) => match express {
                Expression::Literal(lit) => match lit {
                    Literal::Int(int) => Ok(Object::Integer(Integer::new(int))),
                    Literal::Bool(boo) => match boo {
                        true => Ok(Object::Boolean(Boolean::new(true))),
                        false => Ok(Object::Boolean(Boolean::new(false))),
                    },
                    Literal::String(_) => unimplemented!(),
                },
                Expression::PrefixExpression(prefix) => {
                    let expression = self.eval(Node::Expression(*prefix.expression.clone()))?;
                    self.eval_prefix_expression(prefix.prefix_op.clone(), expression)
                },
                Expression::InfixExpression(infix) => {
                    let left = self.eval(Node::Expression(*infix.left_expression.clone()))?;
                    let right = self.eval(Node::Expression(*infix.right_expression.clone()))?;
                    self.eval_infix_expression(infix.infix_op.clone(), left, right)
                },
                Expression::IfExpression(iff) => self.eval_if_expression(iff),
                Expression::Ident(ident) => self.eval_ident_expression(ident),
                Expression::FunctionLiteral(func) => self.eval_function_expression(func),
                Expression::CallExpression(call) => self.eval_call_expression(call),
                express => Err(EvalError::FailedExpression(express)),
            }
        }
    }
    fn eval_program(&mut self, program: Program) -> Result<Object, EvalError> {
        let mut result = Object::Null;
        
        for entry in program.statements {
            result = self.eval(Node::Statement(entry))?;
            //if result = Result Object return the expression and exit
            if let Object::Return(ret) = result {
                return Ok(*ret.value);
            }

        }
        return Ok(result);
    }
    fn eval_prefix_expression(&mut self, operator: PrefixOp, prefix: Object) -> Result<Object, EvalError> {
        match operator {
            PrefixOp::BANG => return self.eval_bang_expresssion(prefix),
            PrefixOp::NEGATIVE => return self.eval_minus_prefix_operator_expression(prefix),
        }
    }
    fn eval_bang_expresssion(&mut self, prefix: Object) -> Result<Object, EvalError> {
        match prefix {
            Object::Boolean(boolean) => match boolean.value {
                true => Ok(Object::Boolean(Boolean::new(false))),
                false => Ok(Object::Boolean(Boolean::new(true))),
            },
            Object::Integer(_) => return Ok(Object::Boolean(Boolean::new(false))),
            _ => return Ok(Object::Null),
        }
    }

    fn eval_minus_prefix_operator_expression(&mut self, express: Object) -> Result<Object, EvalError> {
        if express.obj_type() != "Integer".to_string() {
            return Err(EvalError::UnknownPrefix(express, PrefixOp::NEGATIVE));
        }
        let output = "-".to_string();
        let value = output + &express.inspect() as &str;
        return Ok(Object::Integer(Integer::new(value as String)));
    }
    fn eval_infix_expression(&mut self, infix_op: InfixOp, left: Object, right: Object) -> Result<Object, EvalError> {
        if left.obj_type() != right.obj_type() {
            return Err(EvalError::Mismatched(left, infix_op, right));
        }
        if left.obj_type() == "Integer" && right.obj_type() == "Integer" {
            return self.eval_infix_integer_expression(infix_op, left, right);
        }
        match infix_op {
            InfixOp::COMPARE =>  return self.bool_to_boolean_object(left == right),
            InfixOp::NEQUALS => return self.bool_to_boolean_object(left != right),
            _ => return Err(EvalError::UnknownInfix(left, infix_op, right)),
        }
    }
    fn eval_infix_integer_expression(&mut self, infix_op: InfixOp, left: Object, right: Object) -> Result<Object, EvalError> {
       let lvalue = left.inspect().parse::<i64>().unwrap();
       let rvalue = right.inspect().parse::<i64>().unwrap();
       match infix_op {
           InfixOp::PLUS => return Ok(Object::Integer(Integer::from_num(lvalue + rvalue))),
           InfixOp::SUBTRACT => return Ok(Object::Integer(Integer::from_num(lvalue - rvalue))),
           InfixOp::MULTIPLY => return Ok(Object::Integer(Integer::from_num(lvalue * rvalue))),
           InfixOp::DIVIDE => return Ok(Object::Integer(Integer::from_num(lvalue / rvalue))),
           InfixOp::GREATERTHAN => return self.bool_to_boolean_object(lvalue > rvalue),
           InfixOp::LESSTHAN => return self.bool_to_boolean_object(lvalue < rvalue),
           InfixOp::COMPARE => return self.bool_to_boolean_object(lvalue == rvalue),
           InfixOp::NEQUALS => return self.bool_to_boolean_object(lvalue != rvalue),
        }
    }
    fn bool_to_boolean_object(&mut self, input: bool) -> Result<Object, EvalError> {
        if input {
            return Ok(Object::Boolean(Boolean::new(true)));
        } else {
            return Ok(Object::Boolean(Boolean::new(false)));
        }
    }
    fn eval_block_statement(&mut self, block: BlockStatement) -> Result<Object, EvalError> {
        let mut resul = Object::Null;    
        for i in 0..block.statements.len() {
            resul = self.eval(Node::Statement(block.statements[i].clone()))?;
            // if resul is a Return Object create the object and exit
            if let Object::Return(ret) = resul {
                return Ok(Object::Return(ret));
            }
        }
        return Ok(resul);
    }
    fn eval_let_statement(&mut self, ls: LetStatement) -> Result<Object, EvalError> {
        //eval LetStatement.value to get object
        let value = self.eval(Node::Expression(ls.value))?;
        // set the environment accordingly
        self.environment.borrow_mut().set(ls.name, &value)?;
        Ok(value)
    }
    fn eval_if_expression(&mut self, iff: IfExpression) -> Result<Object, EvalError> {
        let condition = self.eval(Node::Expression(*iff.condition))?;
        let alternative = iff.alternative;
        if self.is_truthy(condition) {
            return self.eval(Node::Statement(Statement::BlockStatement(iff.consequence)));
        } else if let Some(alternative) = alternative {
                self.eval(Node::Statement(Statement::BlockStatement(alternative)))
        } else {
            return Ok(Object::Null);
        }
    }
    fn is_truthy(&mut self, condition: Object) -> bool {
        match condition {
            Object::Null => return false,
            Object::Boolean(boo) => match boo.value {
                true => return true,
                false => return false,
            },
            _ => return true,
        }
    }
    fn eval_ret_statement(&mut self, ret: ReturnStatement) -> Result<Object, EvalError> {
        let result = self.eval(Node::Expression(ret.ret_value))?;
        return Ok(Object::Return(Return::new(result)));
    }
    fn eval_ident_expression(&self, ident: String) -> Result<Object, EvalError> {
        self.environment.borrow_mut().print();
        let value = self.environment.borrow_mut().get(&ident);
        match value {
            Some(value) => Ok(value),
            None => {
                Err(EvalError::UnknownIdent(ident))
            }
        }
    }
    fn eval_function_expression(&mut self, func: FunctionLiteral) -> Result<Object, EvalError> {
        return Ok(
            Object::Function(
                    Function {
                        parameters: func.parameters.clone(),
                        body: func.body.clone(),
                        environment: self.environment.clone(),
                    }
                )
        )
    }
    fn eval_call_expression(&mut self, call: CallExpression) -> Result<Object, EvalError> {
        // CallExpression =(arguments: Arguments) {function: Box<Expression>} 
        // eval the body then eval the expressions
        // then return the object from apply_function
        let function = self.eval(Node::Expression(*call.function))?;
        let args = self.eval_expressions(call.arguments)?;
        return self.apply_function(function, args);
    }
    fn eval_expressions(&mut self, func: Arguments) -> Result<Vec<Object>, EvalError> {
        let mut result = Vec::<Object>::new();
        for i in 0..func.len() {
            let evaluated = self.eval(Node::Expression(func.variables[i].clone()))?;
            result.push(evaluated);
        }
        return Ok(result);
    }
    fn apply_function(&mut self, func: Object, args: Vec<Object>) -> Result<Object, EvalError> {
        // if func is a function object
        // set the env to the current environment
        // create a new environment from the func.environment, set it to our environment.outer
        if let Object::Function(function) = func {
            let env = &self.environment.clone();
            let mut environment = Environment::new_enclosed_environment(Rc::clone(&Rc::new(function.environment)));
            for (entry, parameter) in function.parameters.into_iter().enumerate() {
                //for every parameter get the object from the args, push both into environment
                environment.set(parameter.to_string(), args.get(entry).unwrap())?;
            }
            // set our environment equal to our new environment
            self.environment = RefCell::new(environment);
            // eval func.body then set our environment back to our original environment before
            // returning the newly evaluated environment
            let evaluated = self.eval(Node::Statement(Statement::BlockStatement(function.body)))?;
            self.environment = env.clone();
            return Ok(evaluated);
        } 
        return Err(EvalError::FailedObject(func));
    }
}
#[cfg(test)]
mod test{
    use std::vec;
    use super::{Evalulator, EvalError};

    use crate::token::ast::{Node, InfixOp, PrefixOp};
    use crate::token::object::{Object, ObjectType, Integer, Boolean};
    use crate::token::{parser::Parser, token::Lexer};

    #[test]
    fn test_eval_integer_expression() -> Result<(), EvalError> {
        let input = vec![
            "5",
            "10",
            "-5",
            "-10",
            "5 + 5 + 5 + 5 - 10",
            "2 * 2 * 2 * 2 * 2",
            "-50 + 100 + -50",
            "5 * 2 + 10",
            "5 + 2 * 10",
            "20 + 2 * -10",
            "50 / 2 * 2 + 10",
            "2 * (5 + 10)",
            "3 * 3 * 3 + 10",
            "3 * (3 * 3) + 10",
            "(5 + 10 * 2 + 15 / 3) * 2 + -10",
        ];
        let expected: Vec<i64> = vec![
            5,
            10,
            -5,
            -10,
            10,
            32,
            0,
            20,
            25,
            0,
            60,
            30,
            37,
            37,
            50,
        ];
        
        for i in 0..input.len() {
            let evaluated = test_eval(input[i].to_string())?;
            if !test_integer_object(&evaluated, expected[i]) {
                println!("Iteration: {}", i);
                return Err(EvalError::FailedEval(format!("Expected: {}, got: {}", &expected[i], &evaluated)));
            }
        }
        Ok(())
    }
    fn test_eval(input: String) -> Result<Object, EvalError> {
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        let mut evalulator = Evalulator::new();
        Ok(evalulator.eval(Node::Program(program)))?
    }

    fn test_integer_object(obj: &Object, expected: i64) ->  bool {
        if obj.obj_type() == "Boolean" || obj.obj_type() == "Null" {
            return false;
        }
        if obj.inspect() != expected.to_string() {
            return false;
        }

        return true;
    }
    #[test]
    fn test_boolean_expression() -> Result<(), EvalError> {
        let input = vec![
            "true",
            "false",
            "1 < 2",
            "1 > 2",
            "1 < 1",
            "1 > 1",
            "1 == 1",
            "1 != 1",
            "1 == 2",
            "1 != 2",
            "true == true",
            "false == false",
            "true == false",
            "true != false",
            "false != true",
            "(1 < 2) == true",
            "(1 < 2) == false",
            "(1 > 2) == true",
            "(1 > 2) == false",
        ];
        let expected =vec![
            true,
            false,
            true,
            false,
            false,
            false,
            true,
            false,
            false,
            true,
            true,
            true,
            false,
            true,
            true,
            true,
            false,
            false,
            true,
        ];

        for i in 0..input.len() {
            let evaluated = test_eval(input[i].to_string())?;
            if !test_boolean_object(&evaluated, expected[i]) {
                println!("Iteration: {}", i);
                return Err(EvalError::FailedEval(format!("Expected: {}, got: {}", &expected[i], &evaluated)));
            }
        }
        Ok(())
    }
    fn test_boolean_object(obj: &Object, expected: bool) ->  bool {
        if obj.obj_type() == "Integer" || obj.obj_type() == "Null" {
            return false;
        }
        if obj.inspect() != expected.to_string() {
            return false;
        }

        return true;
    }

    #[test]
    fn test_bang_operator() -> Result<(), EvalError> {
        let input = vec![
            "!true",
            "!false",
            "!5",
            "!!true",
            "!!false",
            "!!5",
        ];
        let expected =vec![
            false,
            true,
            false,
            true,
            false,
            true,
        ];

        for i in 0..input.len() {
            let evaluated = test_eval(input[i].to_string())?;
            if !test_boolean_object(&evaluated, expected[i]) {
                println!("iteration: {}", &i);
                return Err(EvalError::FailedEval(format!("Expected: {}, got: {}", &expected[i], &evaluated)));
            }
        }
        Ok(())
    }
    #[test]
    fn test_if_expressions() -> Result<(), EvalError> {
        let input = vec![
            "if (true){10}",
            "if (false){10}",
            "if (1){10}",
            "if (1<2){10}",
            "if (1>2){10}",
            "if (1>2){10}else{20}",
            "if (1<2){10}else{20}",
        ];
        let expected = vec![
            Object::Integer(Integer::from_num(10)),
            Object::Null,
            Object::Integer(Integer::from_num(10)),
            Object::Integer(Integer::from_num(10)),
            Object::Null,
            Object::Integer(Integer::from_num(20)),
            Object::Integer(Integer::from_num(10)),
        ];
        for i in 0..input.len() {
            let evaluated = test_eval(input[i].to_string())?;
            if !test_object(evaluated.clone(), expected[i].clone()) {
                println!("iteration: {}", &i);
                return Err(EvalError::FailedEval(format!("Expected: {}, got: {}", &expected[i], &evaluated)));
            }
        }
        Ok(())
    }
    fn test_object(evaluated: Object, expected: Object) -> bool {
        if evaluated == expected {
            return true;
        } else {
            return false;
        }
    }
    #[test]
    fn test_return_statements()-> Result<(), EvalError> {
        let input = vec![
            "return 10;",
            "return 10;9;",
            "return 2*5;9;",
            "9;return 2*5;9;",
            "if (10>1) {
                if (10>1){
                    return 10;
                }
                return 1;
            }"
        ];
        let expected = vec![
            10,
            10,
            10,
            10,
            10,
        ];
        for i in 0..input.len() {
            let evaluated = test_eval(input[i].to_string())?;
            if !test_integer_object(&evaluated, expected[i].clone()) {
                println!("iteration: {}", &i);
                return Err(EvalError::FailedEval(format!("Expected: {}, got: {}", &expected[i], &evaluated)));
            }
        }
        Ok(())
    }
    #[test]
    fn test_errors() -> Result<(), EvalError> {
        let input = vec![
            "5 + true;",
            "5 + true; 5;",
            "-true",
            "true + false;",
            "5; true + false; 5",
            "if (10>1) { true + false; }",
        ];
        let expected = vec![
            EvalError::Mismatched(Object::Integer(Integer::from_num(5)), InfixOp::PLUS, Object::Boolean(Boolean::new(true))),
            EvalError::Mismatched(Object::Integer(Integer::from_num(5)), InfixOp::PLUS, Object::Boolean(Boolean::new(true))),
            EvalError::UnknownPrefix(Object::Boolean(Boolean::new(true)), PrefixOp::NEGATIVE),
            EvalError::UnknownInfix(Object::Boolean(Boolean::new(true)), InfixOp::PLUS, Object::Boolean(Boolean::new(false))),
            EvalError::UnknownInfix(Object::Boolean(Boolean::new(true)), InfixOp::PLUS, Object::Boolean(Boolean::new(false))),
            EvalError::UnknownInfix(Object::Boolean(Boolean::new(true)), InfixOp::PLUS, Object::Boolean(Boolean::new(false))),
        ];
        for i in 0..input.len() {
            match test_eval(input[i].to_string()) {
                Ok(error) => {
                    panic!("expected error: {}, got: {}, for iteration: {}", expected[i], error, i);
                }
                Err(error) => assert_eq!(error, expected[i]),
            }

        }
        Ok(())
    }
    #[test]
    fn test_let_statements() -> Result<(), EvalError> {
        let input = vec![
            "let a = 5; a;",
            "let a = 5 * 5; a;",
            "let a = 5; let b = a; b;",
            "let a = 5; let b = a; let c = a + b + 5; c;",
        ];

        let expected = vec![
            5,
            25,
            5,
            15,
        ];
        
        for i in 0..input.len() {
            let evaluated = test_eval(input[i].to_string())?;
            if !test_integer_object(&evaluated, expected[i].clone()) {
                println!("iteration: {}", &i);
                return Err(EvalError::FailedEval(format!("Expected: {}, got: {}", &expected[i], &evaluated)));
            }
        }
        Ok(())
    }
    #[test]
    fn test_environment() -> () {
        let input = vec![
            "let a = 5;",
            "a;",
        ];

        let mut evalulator = Evalulator::new();
        for i in input {
        let l = Lexer::new(i.to_string());
        let mut p = Parser::new(l);

        let program = p.parse_program();
        evalulator.eval(Node::Program(program)).unwrap();
        }
        evalulator.environment.borrow_mut().print();
        let a = "a".to_string();
        if let Some(a) = evalulator.environment.borrow_mut().get(&a) {
        println!("a: {}", a);
        }
        assert!(!evalulator.environment.borrow_mut().is_empty());
    }
    #[test]
    fn test_function_object() -> Result<(), EvalError> {
        let input = vec![
            "fn(x) { x + 2; };",
        ];

        let expected = vec![
           "fn (x) {\n(x + 2)\n}", 
        ];
        for i in 0..input.len() {
            let evaluated = test_eval(input[i].to_string())?; 
            println!("{}", evaluated);
            assert_eq!(evaluated.inspect(), expected[i].to_string());
        }
        Ok(())
    }
    #[test]
    fn test_function_application() -> Result<(), EvalError> {
        let input = vec![
            "let identitiy = fn(x) { x; }; identitiy(5);",
            "let identitiy = fn(x) { return x; }; identitiy(5);",
            "let double = fn(x) { return x * 2; }; double(5);",
            "let add = fn(x, y) { x + y; }; add(5, 5);",
            "let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));",
            "fn(x) { x; }(5)",
        ];

        let expected = vec![
           5,
           5,
           10,
           10,
           20,
           5,
        ];
        for i in 0..input.len() {
            let evaluated = test_eval(input[i].to_string())?; 
            if !test_integer_object(&evaluated, expected[i].clone()) {
                println!("iteration: {}", &i);
                return Err(EvalError::FailedEval(format!("Expected: {}, got: {}", &expected[i], &evaluated)));
            }

        }
        Ok(())
    }
}
