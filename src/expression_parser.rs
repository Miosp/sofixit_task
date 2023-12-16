use indexmap::IndexMap;
use pest::{Parser, pratt_parser::PrattParser, iterators::Pairs};
use pest_derive::Parser;
use lazy_static::lazy_static;

lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};

        PrattParser::new()
            .op(Op::infix(Rule::add, Left) | Op::infix(Rule::subtract, Left))
            .op(Op::infix(Rule::multiply, Left) | Op::infix(Rule::divide, Left))
    };
}

#[derive(Parser)]
#[grammar = "src/expressionGrammar.pest"]
struct ExpressionParser;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Function {
    SquareRoot,
    PowerOf2,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum InfixOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    BinOp(BinOp),
    Number(i64),
    Float(f64),
    String(String),
    Negate(Box<Expression>),
    Constant(String),
    Parenthesis(Box<Expression>),
    Funct(Function, Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinOp {
    pub op: InfixOp,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

impl Expression {
    /// Evaluates the expression and returns the result.
    /// 
    /// # Arguments
    /// 
    /// * `map` - A map containing the values of the constants used in the expression.
    /// 
    /// # Returns
    /// 
    /// The result of the expression. 
    pub fn eval(&self, map: &IndexMap<String, Expression>) -> Result<Expression, String>{
        use Expression::*;

        match self {
            Number(_) => Ok(self.clone()),
            Float(_) => Ok(self.clone()),
            String(_) => Ok(self.clone()),
            Constant(_) => Ok(map.get(self.to_string().as_str()).ok_or(format!("Constant {} not found", self.to_string()))?.clone()),
            Negate(x) => {
                let x = x.eval(map)?;
                match x {
                    Number(n) => Ok(Number(-n)),
                    Float(f) => Ok(Float(-f)),
                    _ => Err(format!("Cannot negate {}", x.to_string())),
                }
            },
            Parenthesis(x) => x.eval(map),
            Funct(func, x) => {
                let x = x.eval(map)?;
                match func {
                    Function::SquareRoot => match x {
                        Number(n) => Ok(Float((n as f64).sqrt())),
                        Float(f) => Ok(Float(f.sqrt())),
                        _ => Err(format!("Cannot take square root of {}", x.to_string())),
                    },
                    Function::PowerOf2 => match x {
                        Number(n) => Ok(Number(n * n)),
                        Float(f) => Ok(Float(f * f)),
                        _ => Err(format!("Cannot square {}", x.to_string())),
                    },
                }
            },
            BinOp(b) => {
                let l = b.left.eval(map)?;
                let r = b.right.eval(map)?;

                match (l, r) {
                    (Number(l), Number(r)) => match b.op {
                        InfixOp::Add => Ok(Number(l + r)),
                        InfixOp::Subtract => Ok(Number(l - r)),
                        InfixOp::Multiply => Ok(Number(l * r)),
                        InfixOp::Divide => Ok(Number(l / r)),
                    },
                    (Float(l), Float(r)) => match b.op {
                        InfixOp::Add => Ok(Float(l + r)),
                        InfixOp::Subtract => Ok(Float(l - r)),
                        InfixOp::Multiply => Ok(Float(l * r)),
                        InfixOp::Divide => Ok(Float(l / r)),
                    },
                    (Number(l), Float(r)) => match b.op {
                        InfixOp::Add => Ok(Float(l as f64 + r)),
                        InfixOp::Subtract => Ok(Float(l as f64 - r)),
                        InfixOp::Multiply => Ok(Float(l as f64 * r)),
                        InfixOp::Divide => Ok(Float(l as f64 / r)),
                    },
                    (Float(l), Number(r)) => match b.op {
                        InfixOp::Add => Ok(Float(l + (r as f64))),
                        InfixOp::Subtract => Ok(Float(l - (r as f64))),
                        InfixOp::Multiply => Ok(Float(l * (r as f64))),
                        InfixOp::Divide => Ok(Float(l / (r as f64))),
                    },
                    (String(l), String(r)) => match b.op {
                        InfixOp::Add => Ok(String(format!("{}{}", l, r))),
                        _ => Err(format!("Cannot perform operation {} on strings", b.op.to_string())),
                    },
                    (String(l), Number(r)) => match b.op {
                        InfixOp::Multiply => Ok(String(l.repeat(r as usize))),
                        _ => Err(format!("Cannot perform operation {} on strings", b.op.to_string())),
                    },
                    (Number(l), String(r)) => match b.op {
                        InfixOp::Multiply => Ok(String(r.repeat(l as usize))),
                        _ => Err(format!("Cannot perform operation {} on strings", b.op.to_string())),
                    },
                    (l, r) => Err(format!("Cannot perform operation {} on {} and {}", b.op.to_string(), l.to_string(), r.to_string())),
                }
            }
        }
    }

    pub fn to_string(&self) -> String {
        use Expression::*;

        match self {
            Number(n) => n.to_string(),
            Float(f) => f.to_string(),
            String(s) => s.clone(),
            Constant(s) => s.clone(),
            Parenthesis(expr) => format!("({})", expr.to_string()),
            Funct(func, expr) => format!("{}({})", match func {
                Function::SquareRoot => "sqrt",
                Function::PowerOf2 => "pow2",
            }, expr.to_string()),
            Negate(expr) => format!("-{}", expr.to_string()),
            BinOp(b) => format!("{} {} {}", b.left.to_string(), match b.op {
                InfixOp::Add => "+",
                InfixOp::Subtract => "-",
                InfixOp::Multiply => "*",
                InfixOp::Divide => "/",
            }, b.right.to_string()),
        }
    }
}

impl InfixOp {
    pub fn to_string(&self) -> String {
        match self {
            InfixOp::Add => "+",
            InfixOp::Subtract => "-",
            InfixOp::Multiply => "*",
            InfixOp::Divide => "/",
        }.to_string()
    }

}

/// Parses the given expression and returns the corresponding `Expression` object.
/// 
/// # Arguments
/// 
/// * `expression` - `&str` format of the expression to parse.
/// 
/// # Returns
/// 
/// The parsed expression.
pub fn parse_expression(expression: &str) -> Result<Expression, String> {
    fn parse(pairs: Pairs<'_, Rule>) -> Result<Expression, String> {
        PRATT_PARSER.map_primary(|primary| match primary.as_rule() {
            Rule::number => Ok(Expression::Number(primary.as_str().parse().unwrap())),
            Rule::string => Ok(Expression::String(primary.as_str().to_string())),
            Rule::constant => Ok(Expression::Constant(primary.as_str().to_string())),
            Rule::parenthesesExpr => parse(primary.into_inner()).map(|expr| Expression::Parenthesis(Box::new(expr))),
            Rule::functionExpr => {
                let mut inner = primary.into_inner();
                let function = match inner.next().unwrap().as_str() {
                    "sqrt" => Function::SquareRoot,
                    "pow2" => Function::PowerOf2,
                    _ => unreachable!(),
                };
                let expr = parse(inner)?;
                Ok(Expression::Funct(function, Box::new(expr)))
            },
            Rule::negated => parse(primary.into_inner()).map(|expr| Expression::Negate(Box::new(expr))),
            //If we reach the end of the expression, we're done
            _ => unreachable!(),
        }).map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::add => InfixOp::Add,
                Rule::subtract => InfixOp::Subtract,
                Rule::multiply => InfixOp::Multiply,
                Rule::divide => InfixOp::Divide,
                _ => unreachable!(),
            };
            if lhs.is_err() || rhs.is_err() {
                return Err(format!("Failed to parse expression: {:?}", lhs.err().unwrap_or(rhs.err().unwrap())));
            }
            Ok(Expression::BinOp(BinOp {
                op,
                left: Box::new(lhs.unwrap()),
                right: Box::new(rhs.unwrap()),
            }))
        })
        .parse(pairs)
    }
    let output = ExpressionParser::parse(Rule::result, expression).map_err(|e| format!("{:?}", e))?;
    parse(output)
}

#[test]
fn test_parse_expression() {
    let expression ="4 * (4 + 6) + -1 + -2 --2 -(-1) -(-4* 55) + sqrt(nice) + pow2(type)";
    let result = parse_expression(expression);
    assert!(result.is_ok());
    println!("{:?}", result.unwrap());
}