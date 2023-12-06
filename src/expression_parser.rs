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
pub struct ExpressionParser;

#[derive(Debug, Clone)]
enum Function {
    SquareRoot,
    PowerOf2,
}

#[derive(Debug, Clone)]
enum InfixOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}
#[derive(Debug, Clone)]
enum Expression {
    BinOp {
        op: InfixOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Number(i64),
    String(String),
    Negate(Box<Expression>),
    Constant(String),
    Parenthesis(Box<Expression>),
    Funct(Function, Box<Expression>),
}

fn parse_expression(expression: &str) -> Result<Expression, String> {
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
            Ok(Expression::BinOp {
                op,
                left: Box::new(lhs.unwrap()),
                right: Box::new(rhs.unwrap()),
            })
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