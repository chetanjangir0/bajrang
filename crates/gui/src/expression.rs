pub fn evaluate(input: &str) -> Result<f64, String> {
    let mut parser = Parser::new(input);
    let value = parser.expression()?;

    parser.skip_whitespace();

    if parser.is_finished() {
        if value.is_finite() {
            Ok(value)
        } else {
            Err("Expression result must be finite.".to_string())
        }
    } else {
        Err(format!(
            "Unexpected token '{}' in expression.",
            parser.current().unwrap_or_default()
        ))
    }
}

struct Parser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    fn expression(&mut self) -> Result<f64, String> {
        let mut value = self.term()?;

        loop {
            self.skip_whitespace();

            if self.consume('+') {
                value += self.term()?;
            } else if self.consume('-') {
                value -= self.term()?;
            } else {
                return Ok(value);
            }
        }
    }

    fn term(&mut self) -> Result<f64, String> {
        let mut value = self.factor()?;

        loop {
            self.skip_whitespace();

            if self.consume('*') {
                value *= self.factor()?;
            } else if self.consume('/') {
                let divisor = self.factor()?;

                if divisor == 0.0 {
                    return Err("Expression cannot divide by zero.".to_string());
                }

                value /= divisor;
            } else {
                return Ok(value);
            }
        }
    }

    fn factor(&mut self) -> Result<f64, String> {
        self.skip_whitespace();

        if self.consume('+') {
            return self.factor();
        }

        if self.consume('-') {
            return Ok(-self.factor()?);
        }

        if self.consume('(') {
            let value = self.expression()?;
            self.skip_whitespace();

            if self.consume(')') {
                return Ok(value);
            }

            return Err("Expression is missing a closing ')'.".to_string());
        }

        self.number()
    }

    fn number(&mut self) -> Result<f64, String> {
        self.skip_whitespace();

        let start = self.position;
        let mut has_digit = false;
        let mut has_decimal = false;

        while let Some(character) = self.current() {
            if character.is_ascii_digit() {
                has_digit = true;
                self.advance(character);
            } else if character == '.' && !has_decimal {
                has_decimal = true;
                self.advance(character);
            } else {
                break;
            }
        }

        if matches!(self.current(), Some('e' | 'E')) && has_digit {
            let exponent_start = self.position;
            self.advance(self.current().unwrap_or_default());

            if matches!(self.current(), Some('+' | '-')) {
                self.advance(self.current().unwrap_or_default());
            }

            let mut has_exponent_digit = false;

            while let Some(character) = self.current() {
                if character.is_ascii_digit() {
                    has_exponent_digit = true;
                    self.advance(character);
                } else {
                    break;
                }
            }

            if !has_exponent_digit {
                self.position = exponent_start;
            }
        }

        if !has_digit {
            return Err("Expression must contain a number.".to_string());
        }

        self.input[start..self.position]
            .parse::<f64>()
            .map_err(|_| "Expression contains an invalid number.".to_string())
    }

    fn consume(&mut self, expected: char) -> bool {
        if self.current() == Some(expected) {
            self.advance(expected);
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(character) = self.current() {
            if character.is_whitespace() {
                self.advance(character);
            } else {
                break;
            }
        }
    }

    fn current(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn advance(&mut self, character: char) {
        self.position += character.len_utf8();
    }

    fn is_finished(&self) -> bool {
        self.position == self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::evaluate;

    const TOLERANCE: f64 = 1.0e-12;

    fn assert_eval(input: &str, expected: f64) {
        let actual = evaluate(input).unwrap();
        assert!(
            (actual - expected).abs() < TOLERANCE,
            "{input} evaluated to {actual}, expected {expected}"
        );
    }

    #[test]
    fn evaluates_basic_arithmetic_with_precedence() {
        assert_eval("1/2", 0.5);
        assert_eval("2 + 3 * 4", 14.0);
        assert_eval("(2 + 3) * 4", 20.0);
        assert_eval("-1 + 2", 1.0);
        assert_eval("1e-3 * 2", 0.002);
    }

    #[test]
    fn rejects_invalid_expressions() {
        assert!(evaluate("").is_err());
        assert!(evaluate("1 / 0").is_err());
        assert!(evaluate("1 +").is_err());
        assert!(evaluate("(1 + 2").is_err());
        assert!(evaluate("1 meter").is_err());
    }
}
