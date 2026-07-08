use crate::{
    ConfigValue,
    parse::{
        Result, SyntaxError,
        parse_utils::{is_whitespace, skip_space_and_comments},
    },
};
use parsey::{Parsey, Spanned, parse_any};

impl<'c> ConfigValue<'c> {
    pub fn parse(parser: &mut Parsey<'c>) -> Result<Option<Spanned<Self>>> {
        skip_space_and_comments(parser, false)?;

        parse_any!(
            parser,
            Self::parse_number,
            Self::parse_bool,
            Self::parse_string,
            Self::parse_ident_string
        )
    }

    fn parse_number(parser: &mut Parsey<'c>) -> Result<Option<Spanned<Self>>> {
        parser.sandbox_result(|parser| {
            let number = parser.take_until_or_end(|c| is_whitespace(c));

            if !number
                .str()
                .starts_with(|c: char| c == '.' || c.is_ascii_digit())
            {
                return Ok(None);
            }

            if number.str().contains('.') {
                let float = number
                    .str()
                    .parse::<f64>()
                    .map_err(|e| SyntaxError::InvalidFloat {
                        span: number.span().into(),
                        parse_error: e,
                    })?;
                Ok(Some(Spanned::new(Self::Float(float), number.span())))
            } else {
                let mut radix = 10;
                let mut num_no_radix = number.fork();
                if let Some(_) = num_no_radix.take("0x") {
                    radix = 16;
                } else if let Some(_) = num_no_radix.take("0b") {
                    radix = 2;
                }

                let int = i64::from_str_radix(num_no_radix.str(), radix).map_err(|e| {
                    SyntaxError::InvalidInt {
                        span: number.span().into(),
                        parse_error: e,
                    }
                })?;

                Ok(Some(Spanned::new(Self::Int(int), number.span())))
            }
        })
    }

    fn parse_bool(parser: &mut Parsey<'c>) -> Result<Option<Spanned<Self>>> {
        parser.sandbox_result(|parser| {
            let ident = parser.take_until_or_end(|c| is_whitespace(c));
            match ident.str() {
                "true" | "#true" => Ok(Some(Self::Bool(true))),
                "false" | "#false" => Ok(Some(Self::Bool(false))),
                _ => Ok(None),
            }
            .map(|v| v.map(|v| Spanned::new(v, ident.span())))
        })
    }

    fn parse_string(parser: &mut Parsey<'c>) -> Result<Option<Spanned<Self>>> {
        parser.sandbox_result(|parser| {
            if let None = parser.take("\"") {
                return Ok(None);
            }
            let str = parser
                .take_until(|c| c == '"')
                .ok_or(SyntaxError::Expected {
                    span: parser.span().into(),
                    expected: "string ending quote (\")",
                })?;
            parser.take_n(1); // ending "

            Ok(Some(Spanned::new(Self::String(str.str()), str.span())))
        })
    }

    fn parse_ident_string(parser: &mut Parsey<'c>) -> Result<Option<Spanned<Self>>> {
        parser.sandbox_result(|parser| {
            let string = parser.take_until_or_end(|c| is_whitespace(c));
            if string.str().len() == 0 {
                return Ok(None);
            }
            for (i, char) in string.str().char_indices() {
                if !char.is_ascii_alphanumeric() && !['_', '-'].contains(&char) {
                    let char_len = char.len_utf8();
                    return Err(SyntaxError::IdentStringFailed {
                        invalid_char: (string.span().start() + i, char_len).into(),
                        char,
                    });
                }
            }
            Ok(Some(Spanned::new(
                Self::String(string.str()),
                string.span(),
            )))
        })
    }
}

#[cfg(test)]
mod test {
    use parsey::{Parsey, Spanned};

    use crate::ConfigValue;

    #[test]
    fn parse_number() {
        assert_eq!(
            ConfigValue::parse_number(&mut Parsey::new("0xff")),
            Ok(Some(Spanned::new(ConfigValue::Int(0xFF), 0..4)))
        );

        assert_eq!(
            ConfigValue::parse_number(&mut Parsey::new("0b0101")),
            Ok(Some(Spanned::new(ConfigValue::Int(0b0101), 0..6)))
        );

        assert_eq!(
            ConfigValue::parse_number(&mut Parsey::new("67")),
            Ok(Some(Spanned::new(ConfigValue::Int(67), 0..2)))
        );

        assert_eq!(
            ConfigValue::parse_number(&mut Parsey::new("1.3")),
            Ok(Some(Spanned::new(ConfigValue::Float(1.3), 0..3)))
        );
    }
}
