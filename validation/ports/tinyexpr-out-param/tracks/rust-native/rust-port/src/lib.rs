#[derive(Debug, Clone, PartialEq)]
pub struct Variable<'a> {
    pub name: &'a str,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Constant(f64),
    Variable { name: String, value: f64 },
    Add(Box<Expr>, Box<Expr>),
    Subtract(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Divide(Box<Expr>, Box<Expr>),
    Negate(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub position: usize,
    pub message: String,
}

pub fn compile(expression: &str, variables: &[Variable<'_>]) -> Result<Expr, ParseError> {
    Parser::new(expression, variables).parse_expression()
}

pub fn eval(expr: &Expr) -> f64 {
    match expr {
        Expr::Constant(value) => *value,
        Expr::Variable { value, .. } => *value,
        Expr::Add(left, right) => eval(left) + eval(right),
        Expr::Subtract(left, right) => eval(left) - eval(right),
        Expr::Multiply(left, right) => eval(left) * eval(right),
        Expr::Divide(left, right) => eval(left) / eval(right),
        Expr::Negate(inner) => -eval(inner),
    }
}

pub fn evaluate(expression: &str, variables: &[Variable<'_>]) -> Result<f64, ParseError> {
    compile(expression, variables).map(|expr| eval(&expr))
}

struct Parser<'a> {
    input: &'a str,
    cursor: usize,
    variables: &'a [Variable<'a>],
}

impl<'a> Parser<'a> {
    fn new(input: &'a str, variables: &'a [Variable<'a>]) -> Self {
        Self {
            input,
            cursor: 0,
            variables,
        }
    }

    fn parse_expression(mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_add_sub()?;
        self.skip_ws();
        if self.cursor == self.input.len() {
            Ok(expr)
        } else {
            Err(self.error("unexpected trailing input"))
        }
    }

    fn parse_add_sub(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_mul_div()?;
        loop {
            self.skip_ws();
            if self.take('+') {
                expr = Expr::Add(Box::new(expr), Box::new(self.parse_mul_div()?));
            } else if self.take('-') {
                expr = Expr::Subtract(Box::new(expr), Box::new(self.parse_mul_div()?));
            } else {
                return Ok(expr);
            }
        }
    }

    fn parse_mul_div(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_factor()?;
        loop {
            self.skip_ws();
            if self.take('*') {
                expr = Expr::Multiply(Box::new(expr), Box::new(self.parse_factor()?));
            } else if self.take('/') {
                expr = Expr::Divide(Box::new(expr), Box::new(self.parse_factor()?));
            } else {
                return Ok(expr);
            }
        }
    }

    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        self.skip_ws();
        if self.take('+') {
            return self.parse_factor();
        }
        if self.take('-') {
            return Ok(Expr::Negate(Box::new(self.parse_factor()?)));
        }
        if self.take('(') {
            let expr = self.parse_add_sub()?;
            self.skip_ws();
            if self.take(')') {
                return Ok(expr);
            }
            return Err(self.error("expected ')'"));
        }
        if self
            .peek()
            .is_some_and(|ch| ch.is_ascii_digit() || ch == '.')
        {
            return self.parse_number();
        }
        if self
            .peek()
            .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        {
            return self.parse_variable();
        }
        Err(self.error("expected expression"))
    }

    fn parse_number(&mut self) -> Result<Expr, ParseError> {
        let start = self.cursor;
        while self
            .peek()
            .is_some_and(|ch| ch.is_ascii_digit() || ch == '.')
        {
            self.bump();
        }
        let value = self.input[start..self.cursor]
            .parse::<f64>()
            .map_err(|_| self.error("invalid number"))?;
        Ok(Expr::Constant(value))
    }

    fn parse_variable(&mut self) -> Result<Expr, ParseError> {
        let start = self.cursor;
        while self
            .peek()
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            self.bump();
        }
        let name = &self.input[start..self.cursor];
        let value = self
            .variables
            .iter()
            .find(|variable| variable.name == name)
            .map(|variable| variable.value)
            .ok_or_else(|| self.error_at(start, "unknown variable"))?;
        Ok(Expr::Variable {
            name: name.to_owned(),
            value,
        })
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.bump();
        }
    }

    fn take(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.cursor..].chars().next()
    }

    fn bump(&mut self) {
        if let Some(ch) = self.peek() {
            self.cursor += ch.len_utf8();
        }
    }

    fn error(&self, message: &str) -> ParseError {
        self.error_at(self.cursor, message)
    }

    fn error_at(&self, cursor: usize, message: &str) -> ParseError {
        ParseError {
            position: cursor + 1,
            message: message.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_arithmetic_with_precedence() {
        assert_eq!(evaluate("1 + 2 * 3", &[]), Ok(7.0));
        assert_eq!(evaluate("(1 + 2) * 3", &[]), Ok(9.0));
    }

    #[test]
    fn evaluates_variables_and_unary_minus() {
        let variables = [Variable {
            name: "x",
            value: 4.0,
        }];
        assert_eq!(evaluate("-x + 10", &variables), Ok(6.0));
    }

    #[test]
    fn reports_unknown_variable_position() {
        assert_eq!(
            evaluate("1 + missing", &[]),
            Err(ParseError {
                position: 5,
                message: "unknown variable".to_owned()
            })
        );
    }
}
