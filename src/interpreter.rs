use crate::lexer::{Token, TokenType};
use crate::parser::{Expr, Operator, Parsed};
use crate::util::error;

use std::f64::consts::PI;
use std::fmt::Display;
use std::{
    collections::HashMap,
    io::{Error, ErrorKind, Result},
};

#[derive(Debug)]
enum InterpreterError {
    UndefinedVariable(String),
    UndefinedFunction(String),
    InvalidArguments(String),
    InvalidListLength,
}

impl From<InterpreterError> for Error {
    fn from(value: InterpreterError) -> Error {
        match value {
            InterpreterError::UndefinedVariable(name) => {
                error!(Other, "Undefined variable: {:?}", name)
            }
            InterpreterError::UndefinedFunction(name) => {
                error!(Other, "Undefined function: {:?}", name)
            }
            InterpreterError::InvalidListLength => error!(Other, "Lists must be same length!"),
            InterpreterError::InvalidArguments(name) => {
                error!(Other, "Invalid arguments for function {:?}!", name)
            }
        }
    }
}

impl Into<Expr> for Data {
    fn into(self) -> Expr {
        match self {
            Data::Float(value) => Expr::from(value),
            Data::List(values) => Expr::List(values.into_iter().map(|data| data.into()).collect()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    Float(f64),
    List(Vec<Data>),
}

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Float(value) => write!(f, "{}", value)?,
            Self::List(datas) => {
                let mut buf = String::from("[");
                for (i, data) in datas.iter().enumerate() {
                    match data {
                        Data::Float(value) => buf.push_str(value.to_string().as_str()),
                        Data::List(_) => buf.push_str(data.to_string().as_str()),
                    };
                    if i != datas.len() - 1 {
                        buf.push_str(", ");
                    }
                }
                buf.push(']');

                write!(f, "{}", buf)?;
            }
        };

        Ok(())
    }
}

fn apply_op(left: Data, right: Data, op: Operator) -> InterpreterResult<Data> {
    if let Data::List(ref values1) = left {
        if let Data::List(ref values2) = right {
            if values1.len() != values2.len() {
                return Err(InterpreterError::InvalidListLength);
            }

            return Ok(Data::List(
                values1
                    .iter()
                    .zip(values2)
                    .map(|(value1, value2)| apply_op(value1.clone(), value2.clone(), op.clone()))
                    .map(|res| res.unwrap_or_else(|err| panic!("Error: {}", Error::from(err))))
                    .collect(),
            ));
        }
    };
    let left_val = match left {
        Data::Float(value1) => value1,
        Data::List(values) => {
            return Ok(Data::List(
                values
                    .iter()
                    .map(|data| {
                        apply_op(data.clone(), right.clone(), op.clone())
                            .unwrap_or_else(|err| panic!("{:?}", err))
                    })
                    .collect(),
            ))
        }
    };
    let right_val = match right {
        Data::Float(value1) => value1,
        Data::List(values) => {
            return Ok(Data::List(
                values
                    .iter()
                    .map(|data| {
                        apply_op(left.clone(), data.clone(), op.clone())
                            .unwrap_or_else(|err| panic!("{:?}", err))
                    })
                    .collect(),
            ))
        }
    };

    Ok(match op {
        Operator::Plus => Data::Float(left_val + right_val),
        Operator::Minus => Data::Float(left_val - right_val),
        Operator::Multi => Data::Float(left_val * right_val),
        Operator::Div => Data::Float(left_val / right_val),
        Operator::Pow => Data::Float(left_val.powf(right_val)),
    })
}

fn apply_func(data: Data, func: fn(f64) -> Data) -> Data {
    match data {
        Data::Float(value) => func(value),
        Data::List(values) => Data::List(
            values
                .into_iter()
                .map(|data| apply_func(data, func))
                .collect(),
        ),
    }
}

type Scope = Vec<String>;

pub struct Interpreter {
    parsed: Vec<Parsed>,
    variables: HashMap<String, Data>,
    functions: HashMap<String, (Vec<String>, Expr)>,
}

type InterpreterResult<T> = std::result::Result<T, InterpreterError>;

impl Interpreter {
    pub fn new(parsed: Vec<Parsed>) -> Self {
        Self {
            parsed,
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    fn get_variable(&self, name: &str) -> Option<Data> {
        match name {
            "PI" => Some(Data::Float(PI)),
            "TAU" => Some(Data::Float(PI * 2.0)),
            "GLR" => Some(Data::Float(1.618_033_988_749_894f64)), // Golden ratio
            _ => {
                let Some(data) = self.variables.get(name) else {
                    return None;
                };

                Some(data.clone())
            }
        }
    }

    fn transform_fn_expr(
        &self,
        (parameters, args): (Vec<String>, Vec<Expr>),
        expr: &Expr,
    ) -> InterpreterResult<Expr> {
        let out: Expr;

        match expr {
            Expr::Ident(name) => {
                for (name_, value) in parameters.iter().zip(args) {
                    if name == name_ {
                        out = value.clone();
                        return Ok(out);
                    }
                }
                if let Some(data) = self.get_variable(name) {
                    return Ok(data.into());
                }
                return Err(InterpreterError::UndefinedVariable(name.to_string()));
            }
            Expr::FunctionCall(name, args) => match name {
                _ => {
                    if let Some((parameters, expr2)) = self.functions.get(name) {
                        out =
                            self.transform_fn_expr((parameters.to_vec(), args.to_vec()), expr2)?;
                    } else {
                        return Err(InterpreterError::UndefinedFunction(name.to_string()));
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
            Expr::List(exprs) => {
                return Ok(Expr::List(
                    exprs
                        .iter()
                        .map(|expr| {
                            self.transform_fn_expr((parameters.clone(), args.clone()), expr)
                                .unwrap_or_else(|err| panic!("{:?}", err))
                        })
                        .collect(),
                ))
            }
            Expr::FloatLiteral(_) | Expr::NegFloatLiteral(_) => out = expr.clone(),
        };

        Ok(out)
    }

    fn evaluate_expr(&self, expr: &Expr) -> InterpreterResult<Data> {
        match expr {
            Expr::Ident(name) => {
                if let Some(data) = self.get_variable(name) {
                    return Ok(data.clone());
                } else {
                    return Err(InterpreterError::UndefinedVariable(name.to_string()));
                }
            }
            Expr::FloatLiteral(value) => Ok(Data::Float(
                value
                    .parse()
                    .map_err(|_| unreachable!("Invalid value: {:?}", value))?,
            )),
            Expr::Expr(left, op, right) => {
                let left = self.evaluate_expr(&left)?;
                let right = self.evaluate_expr(&right)?;
                apply_op(left, right, op.clone())
            }
            Expr::NegFloatLiteral(value) => {
                let value_f64: f64 = value
                    .parse()
                    .map_err(|_| unreachable!("Invalid value: {:?}", value))?;
                Ok(Data::Float(-1.0 * value_f64))
            }
            Expr::FunctionCall(name, args) => match name.as_str() {
                "sin" => {
                    if args.len() > 1 {
                        return Err(InterpreterError::InvalidArguments("sin".to_string()));
                    }
                    let arg = self.evaluate_expr(&args[0])?;
                    Ok(apply_func(arg, |arg| Data::Float(arg.sin())))
                }
                "cos" => {
                    if args.len() > 1 {
                        return Err(InterpreterError::InvalidArguments("cos".to_string()));
                    }
                    let arg = self.evaluate_expr(&args[0])?;
                    Ok(apply_func(arg, |arg| Data::Float(arg.cos())))
                }
                "tan" => {
                    if args.len() > 1 {
                        return Err(InterpreterError::InvalidArguments("tan".to_string()));
                    }
                    let arg = self.evaluate_expr(&args[0])?;
                    Ok(apply_func(arg, |arg| Data::Float(arg.tan())))
                }
                _ => {
                    let Some((parameters, expr)) = self.functions.get(name) else {
                        return Err(InterpreterError::UndefinedFunction(name.to_string()));
                    };

                    if args.len() != parameters.len() {
                        return Err(InterpreterError::InvalidArguments(name.to_string()));
                    }

                    let parsable =
                        self.transform_fn_expr((parameters.to_vec(), args.to_vec()), expr)?;
                    return Ok(self.evaluate_expr(&parsable)?);
                }
            },
            Expr::List(exprs) => {
                let vals = exprs
                    .iter()
                    .map(|expr| {
                        self.evaluate_expr(expr)
                            .unwrap_or_else(|err| panic!("{:?}", err))
                    })
                    .collect();

                Ok(Data::List(vals))
            }
        }
    }

    fn clean_scope(&mut self, scope: Scope) {
        for name in &scope {
            self.variables.remove(name);
            self.functions.remove(name);
        }
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
                                unreachable!("Internal error!");
                            }
                        })
                        .collect();
                    self.functions
                        .insert(f.to_string(), (parameters, expr.clone()));
                    scope.push(f.to_string());
                }
                Parsed::FromLoop(min_expr, max_expr, ident_expr, step_expr, block) => {
                    let min = match self.evaluate_expr(&min_expr)? {
                        Data::Float(value) => value,
                        Data::List(_) => {
                            return Err(error!(Other, "From-to-as-loop cannot contain list"))
                        }
                    };
                    let max = match self.evaluate_expr(&max_expr)? {
                        Data::Float(value) => value,
                        Data::List(_) => {
                            return Err(error!(Other, "From-to-as-loop cannot contain list"))
                        }
                    };
                    let step = match self.evaluate_expr(&step_expr)? {
                        Data::Float(value) => value,
                        Data::List(_) => {
                            return Err(error!(Other, "From-to-as-loop cannot contain list"))
                        }
                    };
                    let Expr::Ident(name) = ident_expr else {
                        unreachable!("Internal error!");
                    };
                    let mut i = min;
                    self.variables.insert(name.to_string(), Data::Float(i));
                    while i <= max {
                        let scope = self.execute_block(block.to_vec())?;
                        self.clean_scope(scope);
                        i += step;
                        if let Some(value) = self.variables.get_mut(&name) {
                            *value = Data::Float(i);
                        }
                    }
                    self.variables.remove(&name);
                }
                Parsed::Block(block) => {
                    let scope = self.execute_block(block)?;
                    self.clean_scope(scope);
                }
                Parsed::ForLoop(ident_expr, list_expr, block) => {
                    let list = match self.evaluate_expr(&list_expr)? {
                        Data::List(datas) => datas,
                        Data::Float(_) => return Err(error!(Other, "Expected list!")),
                    };
                    let Expr::Ident(name) = ident_expr else {
                        return Err(error!(Other, "Expected identefier!"));
                    };
                    self.variables
                        .insert(name.clone(), list.get(0).unwrap().clone());
                    for data in &list[1..] {
                        let scope = self.execute_block(block.clone())?;
                        self.clean_scope(scope);
                        if let Some(value) = self.variables.get_mut(&name) {
                            *value = data.clone();
                        }
                    }
                    self.variables.remove(&name);
                }
                Parsed::Destructuring(left, right) => {
                    let Expr::List(left_exprs) = left else {
                        return Err(error!(Other, "Some error!"));
                    };
                    let Expr::List(right_exprs) = right else {
                        return Err(error!(Other, "Some error!"));
                    };
                    if left_exprs.len() != right_exprs.len() {
                        return Err(error!(Other, "Too few idents in destructor!"));
                    }
                    for (left, right) in left_exprs.iter().zip(right_exprs) {
                        let Expr::Ident(name) = left else {
                            return Err(error!(Other, "Only idents allowed in destructor!"));
                        };

                        let None = self.variables.get(name) else {
                            return Err(error!(Other, "Re-decleration of variable {:?}", name));
                        };

                        let data = self.evaluate_expr(&right)?;
                        self.variables.insert(name.clone(), data);
                    }
                }
                _ => unreachable!("Some error!"),
            }
            current += 1;
        }

        Ok(scope)
    }

    pub fn interpret(&mut self) -> Result<()> {
        let scope = self.execute_block(self.parsed.clone())?;
        self.clean_scope(scope);

        Ok(())
    }
}
