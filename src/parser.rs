use crate::lex::{Lexer, Position, TToken, Token};

pub struct ParserError {
    pub message: String,
    pub start: Position,
    pub end: Position,
}

#[derive(Debug)]
pub struct Source {
    pub _type: String,
    pub start: Position,
    pub end: Position,
    pub body: Vec<Stat>,
}

#[derive(Debug)]
pub enum Stat {
    Text {
        _type: String,
        value: String,
        start: Position,
        end: Position,
    },
    Tag {
        _type: String,
        start: Position,
        end: Position,
        value: Expression,
    },
}

#[derive(Debug)]
pub struct Expression {
    pub _type: String,
    pub property: Value,
    pub arguments: Option<Vec<Argument>>,
}

#[derive(Debug)]
pub enum Value {
    Property(Property),
    Int {
        _type: String,
        start: Position,
        end: Position,
        value: usize,
    },
    // Bool {
    //     _type: String,
    //     start: usize,
    //     end: usize,
    //     value: bool,
    // },
}

#[derive(Debug)]
pub struct Property {
    pub _type: String,
    /// A stack of the call to property.
    ///
    /// "$global.bar.foo" -> vec!["$global", "bar", "foo"]
    pub value: Vec<String>,
    pub start: Position,
    pub end: Position,
}

#[derive(Debug)]
pub struct Argument {
    pub _type: String,
}

// TODO: Make parser fault tolorent (some day)
pub struct Parser {
    // errors: vec![],
    tokens: Vec<Token>,
    pointer: usize,
    end_position: Position,
}

impl Parser {
    pub fn from_source(lex: Lexer) -> Self {
        Parser {
            pointer: 0,
            tokens: lex.tokens,
            end_position: lex.position,
        }
    }

    pub fn parse(&mut self) -> Result<Source, String> {
        let body = self.body()?;
        Ok(Source {
            _type: "Source".to_string(),
            start: Position(0, 0),
            end: self.end_position.clone(),
            body,
        })
    }

    fn body(&mut self) -> Result<Vec<Stat>, String> {
        let mut body = Vec::new();

        while !self.is_at_end() {
            let token_data = self.advance();
            if let Some(token_data) = token_data {
                match token_data.token {
                    TToken::Text(text) => body.push(Stat::Text {
                        _type: "Text".to_string(),
                        value: text,
                        start: token_data.start,
                        end: token_data.end,
                    }),

                    TToken::OpenTag => {
                        let tag = self.tag_expression()?;

                        body.push(Stat::Tag {
                            _type: "Tag".to_string(),
                            start: token_data.start,
                            end: token_data.end,
                            value: tag,
                        })
                    }
                    TToken::CloseTag => body.push(Stat::Text {
                        _type: "Text".to_string(),
                        value: "}".to_string(),
                        start: token_data.start,
                        end: token_data.end,
                    }),
                    _ => {}
                }
            }
        }

        Ok(body)
    }

    fn tag_expression(&mut self) -> Result<Expression, String> {
        // Should stop paring when tag end is met
        let property = self.tag_property()?;

        // TODO: check arguments...
        let _arguments = self.tag_arguments();

        let exp = Expression {
            _type: "Expression".to_string(),
            property,
            // TODO: Parse aruments
            arguments: None,
        };

        // self.visit_ws();
        let next = self.advance();
        if next.is_none() {
            return Err("Expected closing tag (none)".to_string());
        }

        match next.unwrap().token {
            TToken::CloseTag => {}
            _ => {
                return Err("Expected closing tag".to_string());
            }
        }

        Ok(exp)
    }

    fn tag_property(&mut self) -> Result<Value, String> {
        // Stop parsing when tag end is met or argument initalizer is met
        // self.visit_ws();
        let advance_res = self.advance();
        if advance_res.is_none() {
            return Err("Unexpected end".to_string());
        }
        let propery_init_token = advance_res.unwrap();

        // Ex: data.guild.meta.name
        if let TToken::Ident(ident) = propery_init_token.token {
            let mut idents = Vec::new();
            idents.push(ident);

            let next = self.peek();
            if next.is_none() {
                return Ok(Value::Property(Property {
                    _type: "Property".to_string(),
                    value: idents,
                    start: propery_init_token.start,
                    end: propery_init_token.end,
                }));
            }

            match next.unwrap().token {
                TToken::Dot => {
                    self.advance();
                    // aka: ident[WS]idnt
                    // there was 'ws' followed by an ident rather then a 'dot'
                    // let mut hitbad = false;

                    while !self.is_at_end() {
                        let token_data = self.peek();
                        if let Some(token) = token_data {
                            match token.token {
                                TToken::Dot => {
                                    self.advance();
                                }
                                TToken::Ident(id) => {
                                    self.advance();
                                    idents.push(id)
                                }
                                _ => break,
                            };
                        } else {
                            break;
                        }
                    }
                }
                TToken::CloseTag | TToken::Text(_) | TToken::WS => {}
                TToken::ArgumentSeperator | TToken::Ident(_) | TToken::Int(_) | TToken::OpenTag => {
                    return Err("Unexpected token_".to_string());
                }
                // TODO: Advance untill token that isnt ws is met
                // TToken::WS => {
                //     // self.visit_ws();
                //     let after_token = &self.tokens[self.pointer].clone();

                //     if let TToken::Dot = after_token.token {
                //         idents.append(&mut self.tag_property_dot())
                //     } else if let TToken::CloseTag = after_token.token {
                //     } else {
                //         return Err(
                //             format!("Unexpected token. found {:?}", after_token.token).to_string()
                //         );
                //     }
                // }
                TToken::ArgumentInitalizer => {}
            }

            Ok(Value::Property(Property {
                _type: "Property".to_string(),
                value: idents,
                start: propery_init_token.start,
                end: propery_init_token.end,
            }))
        } else {
            Err("Expected Identifyer".to_string())
        }
    }

    fn tag_arguments(&mut self) {
        // Stop parsing when tag end is met
    }

    fn advance(&mut self) -> Option<Token> {
        let token = self.peek();
        self.pointer += 1;
        token
    }

    fn peek(&mut self) -> Option<Token> {
        if self.is_at_end() {
            None
        } else {
            self.visit_ws();
            Some(self.tokens[self.pointer].clone())
        }
    }

    /// Used in side peek to ignore whitespace
    /// whitespace is not used in the parser. It is only used in the formater
    fn visit_ws(&mut self) {
        while !self.is_at_end() {
            let current_token = self.tokens[self.pointer].clone();
            match current_token.token {
                TToken::WS => {
                    self.pointer += 1;
                }
                _ => break,
            };
        }
    }

    fn is_at_end(&self) -> bool {
        self.pointer >= self.tokens.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::Lexer;

    fn parse_base(program: &str) -> Result<Source, String> {
        let mut lex = Lexer::from_source(program);
        let res = lex.scan_tokens();
        if res.is_err() {
            println!("{:?}", res);
            return Err(res.err().unwrap());
        }

        let mut parser = Parser::from_source(lex);
        parser.parse()
    }

    #[test]
    fn tag_property_only() {
        assert!(parse_base("Hello, { guild }").is_ok());
        assert!(parse_base("Hello, { guild . name }").is_ok());
    }

    #[test]
    fn tag_property_errs() {
        // TODO: Should cause err...
        assert!(parse_base("Hello, {guild.}").is_err());
        assert!(parse_base("Hello, {guild..}").is_err());

        assert_eq!(
            format!("{:?}", parse_base("Hello, {guild")),
            "Err(\"Expected closing tag (none)\")"
        );
        assert_eq!(
            format!("{:?}", parse_base("Hello, { {guild}")),
            "Err(\"Expected Identifyer\")"
        );
    }
}
