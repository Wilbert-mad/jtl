use super::utils;

#[derive(Debug, Clone, PartialEq)]
pub enum TToken {
    WS,
    Text(String),
    OpenTag,
    Ident(String),
    // 9999999999999999999 - u64 max
    // 999999999 - u32 max
    Int(u32),
    // Bool(bool),
    String(String),
    Dot,
    ArgumentInitalizer,
    ArgumentSeperator,
    CloseTag,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token: TToken,
    pub start: PPosition,
    pub end: PPosition,
}

/// Position(column, line)
// #[deprecated]
// #[derive(Debug, Clone)]
// pub struct Position(pub usize, pub usize);
// impl Position {
//     fn advance_column(&mut self) {
//         self.1 = 0;
//         self.0 += 1
//     }
//     fn advance_line(&mut self) {
//         self.1 += 1
//     }
//     fn back_line(&mut self) {
//         self.1 -= 1
//     }
//     pub fn to_pointer(&self) -> usize {
//         self.1 + self.0
//     }
// }

#[derive(Debug, Clone)]
pub struct PPosition {
    pub column: usize,
    pub line: usize,
}
impl PPosition {
    fn advance_column(&mut self) {
        self.line = 0;
        self.column += 1
    }
    fn advance_line(&mut self) {
        self.line += 1
    }
    fn back_line(&mut self) {
        self.line -= 1
    }
    // pub fn to_offset() -> usize {}
    #[deprecated]
    pub fn to_pointer(&self) -> usize {
        self.line + self.column
    }
}

#[derive(Debug)]
pub struct Lexer {
    pub tokens: Vec<Token>,
    pub pointer: usize,
    source_chars: Vec<char>,
    pub position: PPosition,
    pub is_text: bool,
}

impl Lexer {
    pub fn from_source(program: &str) -> Self {
        Lexer {
            tokens: Vec::new(),
            pointer: 0,
            source_chars: program.chars().collect(),
            position: PPosition { column: 0, line: 0 },
            is_text: true,
        }
    }

    // TODO: Show position when erroring
    pub fn scan_tokens(&mut self) -> Result<(), String> {
        while !self.is_at_end() {
            let char = self.advance();
            let start = self.position.clone();
            // self.position.advance_line();

            let token: Option<TToken> = {
                if let Some(char) = char {
                    match (self.is_text, char) {
                        (_, ' ' | '\r' | '\t') => {
                            self.position.advance_line();
                            Some(TToken::WS)
                        }
                        (_, '\n') => {
                            self.position.advance_column();
                            Some(TToken::WS)
                        }
                        (_, '{') => {
                            self.is_text = false;
                            self.position.advance_line();
                            Some(TToken::OpenTag)
                        }
                        (_, '}') => {
                            self.is_text = true;
                            self.position.advance_line();
                            Some(TToken::CloseTag)
                        }

                        (false, '.') => {
                            self.position.advance_line();
                            Some(TToken::Dot)
                        }
                        (false, ';') => {
                            self.position.advance_line();
                            Some(TToken::ArgumentSeperator)
                        }
                        (false, '|') => {
                            self.position.advance_line();
                            Some(TToken::ArgumentInitalizer)
                        }
                        // TODO: escapeable string
                        (false, '"') => {
                            self.position.advance_line();
                            let start = self.pointer.clone() - 1;
                            loop {
                                self.position.advance_line();
                                match self.advance() {
                                    Some('"') => break,
                                    Some(_) => {}
                                    None => return Err("Unterminated string".to_string()),
                                }
                            }
                            Some(TToken::String(
                                // Remove starting '"' and end '"'
                                self.source_chars[(start + 1)..(self.pointer - 1)]
                                    .into_iter()
                                    .collect(),
                            ))
                        }

                        (true, text) => {
                            let mut content = String::new();
                            self.position.advance_line();

                            let mut chr: Option<char> = Some(text);
                            while chr.is_some() {
                                let x = chr.unwrap().to_string();
                                content += &x;
                                if self.peek() == Some('{') {
                                    break;
                                }
                                if self.peek() == Some('\n') {
                                    break;
                                }
                                chr = self.advance();
                                self.position.advance_line();
                            }

                            Some(TToken::Text(content))
                        }

                        (false, ch) => {
                            self.position.advance_line();
                            let start = self.pointer.clone() - 1;
                            if utils::is_alpha(ch) {
                                loop {
                                    self.position.advance_line();
                                    match self.advance() {
                                        Some(char) => {
                                            if !utils::is_alpha(char) {
                                                break;
                                            }
                                            continue;
                                        }
                                        None => break,
                                    }
                                }
                                self.position.back_line();
                                self.pointer -= 1;

                                let ident =
                                    self.source_chars[start..self.pointer].into_iter().collect();
                                Some(TToken::Ident(ident))
                            } else if utils::is_digit(ch) {
                                loop {
                                    self.position.advance_line();
                                    match self.advance() {
                                        Some(char) => {
                                            if !utils::is_digit(char) {
                                                break;
                                            }
                                            continue;
                                        }
                                        None => break,
                                    }
                                }
                                self.position.back_line();
                                self.pointer -= 1;

                                let integer_res = (self.source_chars[start..self.pointer]
                                    .into_iter()
                                    .collect::<String>())
                                .parse::<u32>();

                                if integer_res.is_ok() {
                                    Some(TToken::Int(integer_res.unwrap()))
                                } else {
                                    return Err(integer_res.err().unwrap().to_string());
                                }
                            } else {
                                return Err("Unexpected token".to_string());
                            }
                        }
                    }
                } else {
                    None
                }
            };

            // println!("{:?}", token);
            if token.is_some() {
                self.tokens.push(Token {
                    token: token.unwrap(),
                    start,
                    end: self.position.clone(),
                })
            }
        }

        Ok(())
    }

    fn advance(&mut self) -> Option<char> {
        let char = self.peek();
        self.pointer += 1;
        char
    }

    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.source_chars[self.pointer])
        }
    }

    fn is_at_end(&self) -> bool {
        self.pointer >= self.source_chars.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_lex() {
        // let mut lex = Lexer::from_source("\n\n");
        let mut lex = Lexer::from_source(
            "Hello, {

ToPlacement|guild.count}",
        );

        let res = lex.scan_tokens();
        // .expect("Scanner should not fail to parse source");
        println!("{:#?}", res);
        println!("{:#?}", lex);
    }
}
