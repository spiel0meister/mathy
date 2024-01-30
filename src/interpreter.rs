use crate::lexer::{Token, TokenType};
use crate::parser::{Expr, Operator, Parsed};
use crate::util::error;

use std::f64::consts::PI;
use std::{
    collections::HashMap,
    io::{Error, ErrorKind, Result},
};

type Scope = Vec<String>;

pub struct Interpreter {
    parsed: Vec<Parsed>,
    variables: HashMap<String, f64>,
    functions: HashMap<String, (Vec<String>, Expr)>,
}

impl Interpreter {
    pub fn new(parsed: Vec<Parsed>) -> Self {
        Self {
            parsed,
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    fn get_variable(&self, name: &str) -> Option<f64> {
        match name {
            "PI" => Some(PI),
            "TAU" => Some(PI * 2.0),
            "GLR" => Some(1.618_033_988_749_894f64), // Golden ratio
            _ => self.variables.get(name).copied(),
        }
    }

    fn transform_fn_expr(
        &self,
        (parameters, args): (Vec<String>, Vec<Expr>),
        expr: &Expr,
    ) -> Result<Expr> {
        let out: Expr;

        match expr {
            Expr::Ident(_) => out = expr.clone(),
            Expr::Parameter(name) => {
                for (name_, value) in parameters.iter().zip(args) {
                    if name == name_ {
                        out = value.clone();
                        return Ok(out);
                    }
                }
                if let Some(value) = self.get_variable(name) {
                    return Ok(Expr::FloatLiteral(value.to_string()));
                }
                return Err(error!(Other, "Undefiend variable: {:?}", name));
            }
            Expr::FunctionCall(name, args) => match name {
                _ => {
                    if let Some((parameters, expr2)) = self.functions.get(name) {
                        out =
                            self.transform_fn_expr((parameters.to_vec(), args.to_vec()), expr2)?;
                    } else {
                        return Err(error!(Other, "Undefined function: {:?}", name));
                    }
                }
            },
            Expr::Expr(left, op, right) => {
                let left_ =
                    self.transform_fn_expr((parameters.to_vec(), args.to_vec()), left.as_ref())?;
                let right_ =
                    self.transform_fn_expr((parameters.to_vec(), args.to_vec()), right.as_ref())?;
                out = Expr::Expr(Box::new(left_), op.clone(), Box::new(right_));
            }
            Expr::FloatLiteral(_) | Expr::NegFloatLiteral(_) => out = expr.clone(),
        };

        Ok(out)
    }

    fn evaluate_expr(&self, expr: &Expr) -> Result<f64> {
        match expr {
            Expr::Ident(name) => {
                if let Some(value) = self.get_variable(name) {
                    Ok(value.clone())
                } else {
                    return Err(error!(Other, "Undefined variable: {:?}", name));
                }
            }
            Expr::FloatLiteral(value) => Ok(value
                .parse()
                .map_err(|err| error!(InvalidInput, "{}", err))?),
            Expr::Expr(left, op, right) => {
                let left_value = self.evaluate_expr(&left)?;
                let right_value = self.evaluate_expr(&right)?;
                Ok(match op {
                    Operator::Plus => left_value + right_value,
                    Operator::Minus => left_value - right_value,
                    Operator::Multi => left_value * right_value,
                    Operator::Div => left_value / right_value,
                    Operator::Pow => left_value.powf(right_value),
                })
            }
            Expr::NegFloatLiteral(value) => {
                let value_f64: f64 = value
                    .parse()
                    .map_err(|err| error!(InvalidInput, "{}", err))?;
                Ok(-1.0 * value_f64)
            }
            Expr::FunctionCall(name, args) => match name.as_str() {
                "sin" => {
                    if args.len() > 1 {
                        return Err(error!(Other, "Too many arguments for sin!"));
                    }
                    let arg = self.evaluate_expr(&args[0])?;
                    Ok(arg.sin())
                }
                "cos" => {
                    if args.len() > 1 {
                        return Err(error!(Other, "Too many arguments for cos!"));
                    }
                    let arg = self.evaluate_expr(&args[0])?;
                    Ok(arg.cos())
                }
                "tan" => {
                    if args.len() > 1 {
                        return Err(error!(Other, "Too many arguments for tan!"));
                    }
                    let arg = self.evaluate_expr(&args[0])?;
                    Ok(arg.tan())
                }
                _ => {
                    let Some((parameters, expr)) = self.functions.get(name) else {
                        return Err(error!(Other, "Undefined function: {:?}", name));
                    };

                    if args.len() != parameters.len() {
                        return Err(error!(Other, "Parameters incorrect!"));
                    }

                    let parsable =
                        self.transform_fn_expr((parameters.to_vec(), args.to_vec()), expr)?;
                    return Ok(self.evaluate_expr(&parsable)?);
                }
            },
            _ => unreachable!(),
        }
    }

    fn clean_scope(&mut self, scope: Scope) -> Result<()> {
        for name in &scope {
            self.variables.remove(name);
            self.functions.remove(name);
        }

        Ok(())
    }

    fn function_exits(&self, name: &str) -> bool {
        match name {
            "sin" | "cos" | "tan" => true,
            _ => self.functions.get(name).is_some(),
        }
    }

    fn execute_block(&mut self, block: Vec<Parsed>) -> Result<Scope> {
        let mut current = 0usize;
        let mut scope: Scope = Vec::new();
        while block.get(current).is_some() {
            let parsed = block.get(current).unwrap().clone();
            match parsed {
                Parsed::Declaration(Token(TokenType::Ident(name), loc), expr) => {
                    if let Some(_) = self.get_variable(&name) {
                        return Err(error!(
                            Other,
                            "Re-decleration of variable {:?} at {}", name, loc
                        ));
                    }
                    self.variables
                        .insert(name.to_string(), self.evaluate_expr(&expr)?);
                    scope.push(name.to_string());
                }
                Parsed::PrintExpr(expr) => {
                    let value = self.evaluate_expr(&expr)?;
                    println!("{}", value);
                }
                Parsed::FunctionDecleration(Token(TokenType::Ident(f), loc), parameters, expr) => {
                    if self.function_exits(&f) {
                        return Err(error!(
                            Other,
                            "Re-decleration of function {:?} at {}", f, loc
                        ));
                    }
                    let parameters = parameters
                        .iter()
                        .map(|Token(t, _)| {
                            if let TokenType::Ident(name) = t {
                                name.to_string()
                            } else {
                                panic!("Internal error!");
                            }
                        })
                        .collect();
                    self.functions
                        .insert(f.to_string(), (parameters, expr.clone()));
                    scope.push(f.to_string());
                }
                Parsed::FromLoop(min_expr, max_expr, ident_expr, step_expr, block) => {
                    let min = self.evaluate_expr(&min_expr)?;
                    let max = self.evaluate_expr(&max_expr)?;
                    let step = self.evaluate_expr(&step_expr)?;
                    let Expr::Ident(name) = ident_expr else {
                        return Err(error!(Other, "Internal error!"));
                    };
                    let mut i = min;
                    self.variables.insert(name.to_string(), i);
                    while i <= max {
                        let scope = self.execute_block(block.to_vec())?;
                        self.clean_scope(scope)?;
                        i += step;
                        *self.variables.get_mut(&name).unwrap() = i;
                    }
                    self.variables.remove(&name);
                }
                Parsed::Block(block) => {
                    let scope = self.execute_block(block)?;
                    self.clean_scope(scope)?;
                }
                _ => unreachable!("Internal error!"),
            }
            current += 1;
        }

        Ok(scope)
    }

    pub fn interpret(&mut self) -> Result<()> {
        let scope = self.execute_block(self.parsed.clone())?;
        self.clean_scope(scope)?;

        Ok(())
    }
}
