use crate::parser::{Expr, Operator, Parsed};
use crate::util::error;

use std::{
    collections::HashMap,
    io::{Error, ErrorKind, Result},
};

type Scope = Vec<String>;

pub struct Interpreter {
    parsed: Vec<Parsed>,
    variables: HashMap<String, Expr>,
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
                if let Some(expr) = self.variables.get(name) {
                    out = expr.clone();
                    return Ok(out);
                }
                return Err(error!(Other, "Undefiend variable: {:?}", name));
            }
            Expr::FunctionCall(name, args) => {
                if let Some((parameters, expr2)) = self.functions.get(name) {
                    out = self.transform_fn_expr((parameters.to_vec(), args.to_vec()), expr2)?;
                } else {
                    return Err(error!(Other, "Undefined function: {:?}", name));
                }
            }
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
                if let Some(expr2) = self.variables.get(name) {
                    self.evaluate_expr(expr2)
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
                })
            }
            Expr::NegFloatLiteral(value) => {
                let value_f64: f64 = value
                    .parse()
                    .map_err(|err| error!(InvalidInput, "{}", err))?;
                Ok(-1.0 * value_f64)
            }
            Expr::FunctionCall(name, args) => {
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

    fn execute_block(&mut self, block: Vec<Parsed>) -> Result<Scope> {
        let mut current = 0usize;
        let mut scope: Scope = Vec::new();
        while block.get(current).is_some() {
            let parsed = block.get(current).unwrap().clone();
            match parsed {
                Parsed::Declaration(name, expr) => {
                    if let Some(_) = self.variables.get(&name) {
                        return Err(error!(Other, "Re-decleration of variable {:?}", name));
                    }
                    self.variables.insert(name.to_string(), expr.clone());
                    scope.push(name.to_string());
                }
                Parsed::PrintExpr(expr) => {
                    let value = self.evaluate_expr(&expr)?;
                    println!("{}", value);
                }
                Parsed::FunctionDecleration(f, parameters, expr) => {
                    if let Some(_) = self.functions.get(&f) {
                        return Err(error!(Other, "Re-decleration of function {:?}", f));
                    }
                    self.functions
                        .insert(f.to_string(), (parameters.to_vec(), expr.clone()));
                    scope.push(f.to_string());
                }
                Parsed::FromLoop(min_expr, max_expr, ident_expr, block) => {
                    let min = self.evaluate_expr(&min_expr)?;
                    let max = self.evaluate_expr(&max_expr)?;
                    let Expr::Ident(name) = ident_expr else {
                        return Err(error!(Other, "Some error!"));
                    };
                    let mut i = min;
                    self.variables
                        .insert(name.to_string(), Expr::FloatLiteral(min.to_string()));
                    while i <= max {
                        let scope = self.execute_block(block.to_vec())?;
                        self.clean_scope(scope)?;
                        i += 1.0;
                        *self.variables.get_mut(&name).unwrap() = Expr::FloatLiteral(i.to_string());
                    }
                    self.variables.remove(&name);
                }
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
