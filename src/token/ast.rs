use std::fmt::Display;

pub enum Node {
    Program(Program),
    Statement(Statement),
    Expression(Expression),
}

#[derive(Debug,Clone,PartialEq)]
pub enum Statement {
    LetStatement(LetStatement),
    ReturnStatement(ReturnStatement),
    Expression(Expression),
    BlockStatement(BlockStatement),
}
impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::BlockStatement(block) => block.fmt(f),
            Statement::LetStatement(let_) => let_.fmt(f),
            Statement::ReturnStatement(return_) => return_.fmt(f),
            Statement::Expression(expression) => expression.fmt(f),
        }
    }
}
impl Statement {
    pub fn get_statement_name(&self) -> String {
        let name = match self{
            Statement::BlockStatement(block_) => block_.get_expression(),
            Statement::LetStatement(let_) => let_.name.clone(),
            Statement::ReturnStatement(return_) => return_.ret_value.clone().to_string(),
            Statement::Expression(expression) => expression.to_string(),
        };
        return name;
    }
    pub fn get_expression(&self) -> String {
        let expression = match self {
            Statement::BlockStatement(block) => block.get_expression(),
            Statement::LetStatement(let_) => let_.value.to_string(),
            Statement::ReturnStatement(return_) => return_.ret_value.to_string(),
            Statement::Expression(expression) => expression.to_string(),
        };
        return expression;
    }
}
#[derive(Debug,Clone,PartialEq)]
pub struct LetStatement {
    pub name: String,
    pub value: Expression,
}
impl Display for LetStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Name: {}, Value: {}", self.name, self.value)
    }
}

#[derive(Debug,Clone,PartialEq)]
pub struct ReturnStatement {
    pub ret_value: Expression,
}
impl Display for ReturnStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", self.ret_value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    CallExpression(CallExpression),
    Ident(String),
    Int(String),
    Literal(Literal),
    PrefixExpression(PrefixExpression),
    InfixExpression(InfixExpression),
    IfExpression(IfExpression),
    FunctionLiteral(FunctionLiteral),
}
impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::CallExpression(call) => call.fmt(f),
            Expression::FunctionLiteral(function) => function.fmt(f),
            Expression::Ident(string) => string.fmt(f),
            Expression::IfExpression(if_) => if_.fmt(f),
            Expression::Int(num) => write!(f, "{num}"),
            Expression::Literal(literal) => literal.fmt(f),
            Expression::PrefixExpression(prefix) => prefix.fmt(f),
            Expression::InfixExpression(infix) => infix.fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(String),
    String(String),
    Bool(bool),
}
impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Int(value) => write!(f, "{value}"),
            Literal::String(string) => write!(f, "{string}"),
            Literal::Bool(bool) => write!(f, "{bool}"),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct PrefixExpression {
    pub prefix_op: PrefixOp,
    // use a box on right expression to prevent infinite recursion of Expressions
    pub expression: Box<Expression>,
}
impl Display for PrefixExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.prefix_op, self.expression)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrefixOp{
    BANG,
    NEGATIVE,
}
impl Display for PrefixOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrefixOp::BANG => write!(f, "!"),
            PrefixOp::NEGATIVE => write!(f, "-"),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct InfixExpression {
    pub left_expression: Box<Expression>,
    pub infix_op: InfixOp,
    pub right_expression: Box<Expression>,
}
impl Display for InfixExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})",
            self.left_expression, self.infix_op, self.right_expression)
    }
}

impl Display for InfixOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InfixOp::COMPARE => write!(f, "=="),
            InfixOp::NEQUALS => write!(f, "!="),
            InfixOp::DIVIDE => write!(f, "/"),
            InfixOp::GREATERTHAN => write!(f, ">"),
            InfixOp::LESSTHAN => write!(f, "<"),
            InfixOp::PLUS => write!(f, "+"),
            InfixOp::MULTIPLY => write!(f, "*"),
            InfixOp::SUBTRACT => write!(f, "-"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InfixOp {
    COMPARE,
    NEQUALS,
    DIVIDE,
    GREATERTHAN,
    LESSTHAN,
    PLUS,
    MULTIPLY,
    SUBTRACT,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpression {
    pub condition: Box<Expression>,
    pub consequence: BlockStatement,
    pub alternative: Option<BlockStatement>,
}
impl Display for IfExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IF {} {{ {} }}", self.condition, self.consequence)?;
        if let Some(alt) = &self.alternative {
            write!(f, " ELSE {{ {} }}", alt)?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionLiteral {
    pub parameters: Parameters,
    pub body: BlockStatement,
}
impl Display for FunctionLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FN ({}) {{ {} }}", self.parameters, self.body)?;
        Ok(())
    }
}
impl Default for FunctionLiteral {
    fn default() -> Self {
        FunctionLiteral {
            parameters: Parameters::default(),
            body: BlockStatement::default(),
        }
    }
}
#[derive(Debug,Clone,PartialEq)]
pub struct Parameters {
    pub variables: Vec<Expression>,
}
impl Default for Parameters {
    fn default() -> Self {
        Parameters {
            variables: Vec::<Expression>::new()
        }
    }
}
impl Display for Parameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        for entry in &self.variables {
            output.push_str(&entry.to_string());
            output.push_str(",");
        }
        output.pop();
        write!(f, "{output}")
    }
}
impl Parameters {
    pub fn is_empty(&self) -> bool {
        if self.variables.len() > 0 {
            return false;
        }
        return true;
    }
}
impl IntoIterator for Parameters {
    type Item = Expression;
    type IntoIter = <Vec<Expression> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.variables.into_iter()
    }
}

#[derive(Debug,Clone,PartialEq)]
pub struct Arguments {
    pub variables: Vec<Expression>,
}
impl Default for Arguments {
    fn default() -> Self {
        Arguments {
            variables: Vec::<Expression>::new()
        }
    }
}
impl Display for Arguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        for entry in &self.variables {
            output.push_str(&entry.to_string());
            output.push_str(",");
        }
        output.pop();
        write!(f, "{output}")
    }
}
impl Arguments {
    pub fn len(&self) -> usize {
        return self.variables.len();
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct CallExpression {
    pub function: Box<Expression>,
    pub arguments: Arguments,
}
impl Display for CallExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.arguments.is_empty() {
            let mut output = format!("FN ({}) {{", self.function);
            for entry in 0..self.arguments.len() {
                output.push_str(&self.arguments.variables[entry].to_string().clone());
                output.push_str(",");
            }
            output.pop();
            output.push_str("}");
            write!(f, "{output}")?;
        } else {
            write!(f, "FN ({})", self.function)?;
        };
        Ok(())
    }
}
impl Arguments {
    pub fn is_empty(&self) -> bool {
        if self.variables.len() > 0 {
            return false;
        }
        return true;
    }
}
pub struct Program {
    pub statements: Vec<Statement>,
}
impl Default for Program {
    fn default() -> Self {
        Program {
            statements: Vec::<Statement>::new(),
        }
    }
}
impl Program {
    pub fn new() -> Program {
       Program::default() 
    }
}
impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for entry in &self.statements {
            entry.fmt(f)?;
        }
        Ok(())
    }
}
#[derive(Debug,Clone,PartialEq)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
}
impl Default for BlockStatement {
    fn default() -> Self {
        BlockStatement {
            statements: Vec::<Statement>::new()
        }
    }
}
impl Display for BlockStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for entry in &self.statements {
            entry.fmt(f)?;
        }
        Ok(())
    }
}

impl BlockStatement {
    fn get_expression(&self) -> String {
        return self.statements[0].get_expression();
    }
}
