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
    pub property: Option<PValue>,
    pub arguments: Option<Vec<Arg>>,
}

#[derive(Debug)]
pub enum PValue {
    Property(Property),
    String {
        _type: String,
        start: Position,
        end: Position,
        value: String,
    },
    Int {
        _type: String,
        start: Position,
        end: Position,
        // Lexer; Int(u32)
        value: u32,
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
pub enum Arg {
    Single(Argument),
    //// Valid -> { toPlacement | (toInt | guild.count ; 0) ; false }
    // Group(Expression),
}

// Invalid -> { toPlacement | (toInt | guild.count ; 0) ; false }
// Valid -> { toPlacement | 0 ; false }
#[derive(Debug)]
pub struct Argument {
    pub _type: String,
    pub value: PValue,
    pub start: Position,
    pub end: Position,
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

pub struct Parser {
    // errors: vec![],
    tokens: Vec<Token>,
    pointer: usize,
    end_position: Position,
}

impl Parser {
    pub fn from_lexer(lex: Lexer) -> Self {
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

        let arguments = self.tag_arguments(&mut errors);

        let exp = Expression {
            _type: "Expression".to_string(),
            property,
            arguments,
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

    fn tag_property(&mut self, errors: &mut Vec<ParserError>) -> Option<PValue> {
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
                                // {idnt.something ""} -> not allowed
                                TToken::String(_) => {
                                    self.advance();
                                    errors.push(ParserError {
                                        message: "Unexpected String".to_string(),
                                        start: token_safe.start,
                                        end: token_safe.end,
                                    })
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
                TToken::ArgumentSeperator
                | TToken::String(_)
                | TToken::Ident(_)
                | TToken::Int(_)
                | TToken::OpenTag => {
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

            Some(PValue::Property(Property {
                _type: "Property".to_string(),
                value: idents,
                start: propery_init_token.start,
                // TODO: Fix, Tecnically not right, the is the end of the first identifyer
                // but fillowing identifyers are not taken into account
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

    fn tag_arguments(&mut self, mut errors: &mut Vec<ParserError>) -> Option<Vec<Arg>> {
        let peek_res = self.peek();
        if peek_res.is_none() {
            let last_token = self.tokens[self.tokens.len() - 1].clone();
            errors.push(ParserError {
                message: "Unexpected EOF".to_string(),
                start: last_token.start,
                end: last_token.end,
            });
            return None;
        }
        let argument_init_token = peek_res.unwrap();

        if argument_init_token.token == TToken::ArgumentInitalizer {
            self.advance();
            let mut arguments: Vec<Arg> = Vec::new();

            let mut expect_seperator = false;
            while !self.is_at_end() {
                let next_token_data = self.peek();
                if next_token_data.is_none() {
                    let last_token = self.tokens[self.tokens.len() - 1].clone();
                    errors.push(ParserError {
                        message: "Unexpected EOF expected Arg".to_string(),
                        start: last_token.start,
                        end: last_token.end,
                    });
                    return None;
                }

                let next_token = next_token_data.unwrap();
                match next_token.token {
                    TToken::String(text) => {
                        self.advance();
                        if expect_seperator {
                            errors.push(ParserError {
                                message: "Expected ';'".to_string(),
                                start: next_token.start.clone(),
                                end: next_token.start.clone(),
                            })
                        }
                        arguments.push(Arg::Single(Argument {
                            _type: "ArgSingle".to_string(),
                            value: PValue::String {
                                _type: "String".to_string(),
                                start: next_token.start.clone(),
                                end: next_token.end.clone(),
                                value: text,
                            },
                            start: next_token.start.clone(),
                            end: next_token.end.clone(),
                        }));
                        expect_seperator = true;
                    }
                    TToken::Int(int) => {
                        self.advance();
                        if expect_seperator {
                            errors.push(ParserError {
                                message: "Expected ';'".to_string(),
                                start: next_token.start.clone(),
                                end: next_token.start.clone(),
                            })
                        }
                        arguments.push(Arg::Single(Argument {
                            _type: "ArgSingle".to_string(),
                            value: PValue::Int {
                                _type: "Int".to_string(),
                                start: next_token.start.clone(),
                                end: next_token.end.clone(),
                                value: int,
                            },
                            start: next_token.start,
                            end: next_token.end,
                        }));
                        expect_seperator = true;
                    }
                    TToken::Ident(ident) => {
                        self.advance();
                        if expect_seperator {
                            errors.push(ParserError {
                                message: "Expected ';'".to_string(),
                                start: next_token.start.clone(),
                                end: next_token.start.clone(),
                            })
                        }
                        let idents_property = self.tag_arg_construct_ident(
                            next_token.start.clone(),
                            next_token.end.clone(),
                            ident,
                            &mut errors,
                        );

                        arguments.push(Arg::Single(Argument {
                            _type: "ArgSingle".to_string(),
                            start: idents_property.start.clone(),
                            end: idents_property.end.clone(),
                            value: PValue::Property(idents_property),
                        }));
                        expect_seperator = true;
                    }
                    TToken::ArgumentSeperator => {
                        self.advance();
                        expect_seperator = false;
                    }
                    TToken::ArgumentInitalizer | TToken::Dot => {
                        self.advance();
                        errors.push(ParserError {
                            message: "Unexpected Token".to_string(),
                            start: next_token.start,
                            end: next_token.end,
                        });
                    }
                    TToken::Text(_) | TToken::WS | TToken::OpenTag => {}
                    TToken::CloseTag => break,
                }
            }

            if arguments.len() > 0 {
                Some(arguments)
            } else {
                None
            }
        } else {
            None
        }
    }

    // {toPlacement|data.guild.meta.name;}
    //             {^^^^^^^^^^^^^^^^^^^^}
    // NOTE: Similar to 'tag_property' idents parsing but not the same
    fn tag_arg_construct_ident(
        &mut self,
        token_start: Position,
        token_end: Position,
        inital: String,
        errors: &mut Vec<ParserError>,
    ) -> Property {
        let mut idents: Vec<String> = Vec::new();
        idents.push(inital);

        let mut last_was_dot = false;
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
                            last_was_dot = true
                        }
                        self.advance();
                    }
                    TToken::Ident(ident) => {
                        self.advance();
                        if !last_was_dot {
                            let end_token = self
                                .advance_until(vec![TToken::ArgumentSeperator, TToken::CloseTag]);

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
                            idents.push(ident);
                            last_was_dot = false;
                        }
                    }
                    TToken::String(_) => {
                        self.advance();
                        errors.push(ParserError {
                            message: "Unexpected Token".to_string(),
                            start: token_safe.start,
                            end: token_safe.end,
                        })
                    }
                    TToken::ArgumentInitalizer | TToken::Int(_) => {}
                    TToken::Text(_) | TToken::WS | TToken::OpenTag => {}
                    TToken::CloseTag => break,
                    TToken::ArgumentSeperator => break,
                };
            } else {
                break;
            }
        }

        Property {
            _type: "Property".to_string(),
            value: idents,
            start: token_start,
            end: token_end,
        }
    }

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
            if self.is_at_end() {
                return None;
            }
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

        let mut parser = Parser::from_lexer(lex);
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
        // NOTE: Not parsed as would like but good enough.
        // after the tag parse of 'tag_property' ends and ['expected identifyer', 'expected "}"'] are errors returned
        // *this is not a bug, but a feature :>jk
        //// -> println!("{:#?}", parse_base("h{.guild}").unwrap());

        // println!("{:#?}", parse_base("h{ {guild").unwrap());
        // println!("{:#?}", parse_base("{ t { } }").unwrap());
    }

    #[test]
    fn tag_arguments_single() {
        // println!("{:#?}", parse_base("{t|guild}").unwrap());
        println!("{:#?}", parse_base("test \n {g}").unwrap());
    }
}
