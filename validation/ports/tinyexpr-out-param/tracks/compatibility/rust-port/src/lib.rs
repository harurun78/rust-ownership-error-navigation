#[derive(Debug, Clone, PartialEq)]
pub struct TeVariable<'a> {
    pub name: &'a str,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TeExpr {
    Constant(f64),
    Variable { name: String, value: f64 },
    Add(Box<TeExpr>, Box<TeExpr>),
    Subtract(Box<TeExpr>, Box<TeExpr>),
    Multiply(Box<TeExpr>, Box<TeExpr>),
    Divide(Box<TeExpr>, Box<TeExpr>),
    Negate(Box<TeExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub position: usize,
    pub message: String,
}

pub fn te_compile<'a>(
    expression: &str,
    variables: &[TeVariable<'a>],
    error: &mut usize,
) -> Option<TeExpr> {
    let mut parser = Parser::new(expression, variables);
    match parser.parse_expression() {
        Ok(expr) => {
            *error = 0;
            Some(expr)
        }
        Err(parse_error) => {
            *error = parse_error.position;
            None
        }
    }
}

pub fn te_eval(expr: &TeExpr) -> f64 {
    match expr {
        TeExpr::Constant(value) => *value,
        TeExpr::Variable { value, .. } => *value,
        TeExpr::Add(left, right) => te_eval(left) + te_eval(right),
        TeExpr::Subtract(left, right) => te_eval(left) - te_eval(right),
        TeExpr::Multiply(left, right) => te_eval(left) * te_eval(right),
        TeExpr::Divide(left, right) => te_eval(left) / te_eval(right),
        TeExpr::Negate(inner) => -te_eval(inner),
    }
}

pub fn te_interp<'a>(expression: &str, variables: &[TeVariable<'a>], error: &mut usize) -> f64 {
    match te_compile(expression, variables, error) {
        Some(expr) => te_eval(&expr),
        None => f64::NAN,
    }
}

struct Parser<'a> {
    input: &'a str,
    cursor: usize,
    variables: &'a [TeVariable<'a>],
}

impl<'a> Parser<'a> {
    fn new(input: &'a str, variables: &'a [TeVariable<'a>]) -> Self {
        Self {
            input,
            cursor: 0,
            variables,
        }
    }

    fn parse_expression(&mut self) -> Result<TeExpr, ParseError> {
        let expr = self.parse_add_sub()?;
        self.skip_ws();
        if self.cursor == self.input.len() {
            Ok(expr)
        } else {
            Err(self.error("unexpected trailing input"))
        }
    }

    fn parse_add_sub(&mut self) -> Result<TeExpr, ParseError> {
        let mut expr = self.parse_mul_div()?;
        loop {
            self.skip_ws();
            if self.take('+') {
                expr = TeExpr::Add(Box::new(expr), Box::new(self.parse_mul_div()?));
            } else if self.take('-') {
                expr = TeExpr::Subtract(Box::new(expr), Box::new(self.parse_mul_div()?));
            } else {
                return Ok(expr);
            }
        }
    }

    fn parse_mul_div(&mut self) -> Result<TeExpr, ParseError> {
        let mut expr = self.parse_factor()?;
        loop {
            self.skip_ws();
            if self.take('*') {
                expr = TeExpr::Multiply(Box::new(expr), Box::new(self.parse_factor()?));
            } else if self.take('/') {
                expr = TeExpr::Divide(Box::new(expr), Box::new(self.parse_factor()?));
            } else {
                return Ok(expr);
            }
        }
    }

    fn parse_factor(&mut self) -> Result<TeExpr, ParseError> {
        self.skip_ws();
        if self.take('+') {
            return self.parse_factor();
        }
        if self.take('-') {
            return Ok(TeExpr::Negate(Box::new(self.parse_factor()?)));
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

    fn parse_number(&mut self) -> Result<TeExpr, ParseError> {
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
        Ok(TeExpr::Constant(value))
    }

    fn parse_variable(&mut self) -> Result<TeExpr, ParseError> {
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
        Ok(TeExpr::Variable {
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
    fn compatibility_interprets_arithmetic_with_variables() {
        let variables = [TeVariable {
            name: "x",
            value: 4.0,
        }];
        let mut error = 999;
        assert_eq!(te_interp("1 + x * 2", &variables, &mut error), 9.0);
        assert_eq!(error, 0);
    }

    #[test]
    fn compatibility_preserves_parentheses_and_unary_minus() {
        let variables = [TeVariable {
            name: "x",
            value: 4.0,
        }];
        let mut error = 999;
        assert_eq!(te_interp("-(x + 2) * 3", &variables, &mut error), -18.0);
        assert_eq!(error, 0);
    }

    #[test]
    fn compatibility_reports_error_position_through_out_param() {
        let mut error = 0;
        assert_eq!(te_compile("1 + missing", &[], &mut error), None);
        assert_eq!(error, 5);
    }
}
