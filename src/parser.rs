use crate::lex::{Lexer, Position, TToken, Token};

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
    pub property: Option<Value>,
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

#[derive(Debug)]
pub struct ParserError {
    pub message: String,
    pub start: Position,
    pub end: Position,
}

#[derive(Debug)]
pub struct ParserResults {
    pub errors: Vec<ParserError>,
    pub ast: Source,
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

    pub fn parse(&mut self) -> ParserResults {
        let mut errors = Vec::new();
        let body = self.body(&mut errors);

        ParserResults {
            errors,
            ast: Source {
                _type: "Source".to_string(),
                start: Position(0, 0),
                end: self.end_position.clone(),
                body,
            },
        }
    }

    fn body(&mut self, mut errors: &mut Vec<ParserError>) -> Vec<Stat> {
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
                        let tag = self.tag_expression(&mut errors);

                        body.push(Stat::Tag {
                            _type: "Tag".to_string(),
                            start: token_data.start,
                            end: token_data.end, // TODO: fix not the real end
                            value: tag,
                        })
                    }
                    // TToken::CloseTag => body.push(Stat::Text {
                    //     _type: "Text".to_string(),
                    //     value: "}".to_string(),
                    //     start: token_data.start,
                    //     end: token_data.end,
                    // }),
                    t => {
                        println!("TODO: {:?}", t);
                    }
                }
            }
        }

        body
    }

    fn tag_expression(&mut self, mut errors: &mut Vec<ParserError>) -> Expression {
        let property = self.tag_property(&mut errors);

        // TODO: check arguments...
        let _arguments = self.tag_arguments();

        let exp = Expression {
            _type: "Expression".to_string(),
            property,
            arguments: None,
        };

        // Hello {guild
        let next = self.advance();
        if next.is_none() {
            let last_token = self.tokens[self.tokens.len() - 1].clone();
            errors.push(ParserError {
                message: "Unexpected EOF expected '}'".to_string(),
                start: last_token.end.clone(),
                end: last_token.end.clone(),
            });
            return exp;
        }

        match next.unwrap().token {
            TToken::CloseTag => {}
            // {toString | 0 world
            //              ^ forgot to close tag but not the end of the file
            _ => {
                let last_token = self.tokens[self.tokens.len() - 1].clone();
                errors.push(ParserError {
                    message: "Expected '}'".to_string(),
                    start: last_token.end.clone(),
                    end: last_token.end.clone(),
                });
            }
        }

        exp
    }

    fn tag_property(&mut self, errors: &mut Vec<ParserError>) -> Option<Value> {
        let peek_res = self.peek();
        if peek_res.is_none() {
            let last_token = self.tokens[self.tokens.len() - 1].clone();
            errors.push(ParserError {
                message: "Unexpected EOF expected Property".to_string(),
                start: last_token.end.clone(),
                end: last_token.end.clone(),
            });
            return None;
        }
        self.advance();
        let propery_init_token = peek_res.unwrap();

        // Ex: data.guild.meta.name
        if let TToken::Ident(ident) = propery_init_token.token {
            let mut idents = Vec::new();
            idents.push(ident);

            let next = self.peek();
            if next.is_none() {
                // NOTE: Should be an error of unclosed or unexpected EOF which is hanndled by 'tag_expression'
                return None;
            }
            let next_token_data = next.unwrap();

            match next_token_data.token {
                TToken::Dot => {
                    self.advance();

                    let mut last_was_dot = true;
                    while !self.is_at_end() {
                        let token_data = self.peek();
                        if let Some(token_safe) = token_data {
                            match token_safe.token {
                                TToken::Dot => {
                                    if last_was_dot {
                                        errors.push(ParserError {
                                            message: "Unexpected '.'".to_string(),
                                            start: token_safe.start,
                                            end: token_safe.end,
                                        });
                                    } else {
                                        last_was_dot = true;
                                    }
                                    self.advance();
                                }
                                TToken::ArgumentSeperator | TToken::Int(_) => {
                                    errors.push(ParserError {
                                        message: "Unexpected Token".to_string(),
                                        start: token_safe.start,
                                        end: token_safe.end,
                                    });
                                    self.advance();
                                }
                                TToken::Ident(idnt) => {
                                    self.advance();
                                    if !last_was_dot {
                                        // // Unsure if we should have the same recovery behaver as with the first instance of just skiping...
                                        let end_token = self.advance_until(vec![
                                            TToken::ArgumentInitalizer,
                                            TToken::CloseTag,
                                        ]);

                                        let end_position = {
                                            if end_token.is_some() {
                                                end_token.unwrap().end
                                            } else {
                                                token_safe.end
                                            }
                                        };

                                        errors.push(ParserError {
                                            message: "Unexpected Token".to_string(),
                                            start: token_safe.start,
                                            end: end_position,
                                        });
                                    } else {
                                        idents.push(idnt);
                                        last_was_dot = false;
                                    }
                                }
                                TToken::WS | TToken::OpenTag | TToken::Text(_) => {}
                                TToken::ArgumentInitalizer | TToken::CloseTag => break, //  _ => break,
                            };
                        } else {
                            break;
                        }
                    }

                    // Aka: {Idnt.} - no follow up was provided
                    if idents.len() < 2 {
                        errors.push(ParserError {
                            message: "Expected Idnt".to_string(),
                            start: next_token_data.start,
                            end: next_token_data.end,
                        });
                    }
                }
                // "TToken::ArgumentInitalizer"{Idnt|...} - should return the ident collected
                TToken::ArgumentInitalizer | TToken::CloseTag | TToken::Text(_) | TToken::WS => {}
                TToken::ArgumentSeperator | TToken::Ident(_) | TToken::Int(_) | TToken::OpenTag => {
                    self.advance();
                    let end_token =
                        self.advance_until(vec![TToken::ArgumentInitalizer, TToken::CloseTag]);

                    let end_position = {
                        if end_token.is_some() {
                            end_token.unwrap().end
                        } else {
                            next_token_data.end
                        }
                    };

                    errors.push(ParserError {
                        message: "Unexpected Token".to_string(),
                        start: next_token_data.start,
                        end: end_position,
                    });
                }
            }

            Some(Value::Property(Property {
                _type: "Property".to_string(),
                value: idents,
                start: propery_init_token.start,
                end: propery_init_token.end,
            }))
        } else {
            errors.push(ParserError {
                message: "Expected Identifyer".to_string(),
                start: propery_init_token.start.clone(),
                end: propery_init_token.start,
            });
            None
        }
    }

    fn tag_arguments(&mut self) {}

    // returns the final token
    fn advance_until(&mut self, skip_until: Vec<TToken>) -> Option<Token> {
        if skip_until.len() > 0 {
            while !self.is_at_end() {
                let peeked = self.peek();
                // none only if it's the end of the program, so should not happen
                if peeked.is_none() {
                    break;
                }

                if skip_until.contains(&peeked.as_ref().unwrap().token) {
                    return Some(peeked.unwrap());
                }
                self.advance();
            }

            None
        } else {
            None
        }
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

    fn parse_base(program: &str) -> Result<ParserResults, String> {
        let mut lex = Lexer::from_source(program);
        let res = lex.scan_tokens();
        if res.is_err() {
            println!("{:?}", res);
            return Err(res.err().unwrap());
        }

        let mut parser = Parser::from_source(lex);
        Ok(parser.parse())
    }

    #[test]
    fn tag_property_only() {
        assert!(parse_base("h{ guild }").unwrap().errors.len() == 0);
        assert!(parse_base("h{ guild . name }").unwrap().errors.len() == 0);
    }

    #[test]
    fn tag_property_errs() {
        assert!(parse_base("h{guild.}").unwrap().errors.len() > 0);
        assert!(parse_base("h{guild..}").unwrap().errors.len() > 0);
        assert!(parse_base("h{guild").unwrap().errors.len() > 0);

        assert!(parse_base("h{ {guild").unwrap().errors.len() > 0);
        // println!("{:#?}", parse_base("h{ {guild").unwrap());
        // println!("{:#?}", parse_base("{ t { } }").unwrap());
    }
}
