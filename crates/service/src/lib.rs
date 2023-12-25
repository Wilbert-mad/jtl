use std::collections::HashMap;

use document::Document;
use jtl_parser::{
    lex::Lexer,
    parser::{Parser, ParserResults, Source},
};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionList, Diagnostic, DiagnosticSeverity, Position,
    Range,
};

use crate::parser_get_node_at::{get_node_at_offset, Node};
pub mod document;
pub mod parser_get_node_at;

#[deprecated]
/// Run diagnostic on source and returns errors
pub fn diagnostic(source: String) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut lexr = Lexer::from_source(&source);
    let lexr_res = lexr.scan_tokens();
    if lexr_res.is_err() {
        let err_msg = lexr_res.err().unwrap();
        diagnostics.push(Diagnostic {
            range: Range {
                start: Position {
                    character: lexr.position.line as u32,
                    line: lexr.position.column as u32,
                },
                end: Position {
                    character: (lexr.position.line + 1) as u32,
                    line: lexr.position.column as u32,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            message: err_msg,
            ..Default::default()
        });
        return diagnostics;
    }

    let mut parser = Parser::from_lexer(lexr);
    for err in parser.parse().errors {
        diagnostics.push(Diagnostic {
            range: Range {
                start: Position {
                    character: err.start.line as u32,
                    line: err.start.column as u32,
                },
                end: Position {
                    character: err.end.line as u32,
                    line: err.end.column as u32,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            message: err.message,
            ..Default::default()
        });
    }

    diagnostics
}

// #[derive(Clone, Debug)]
// /// StructuresUpper(key, value) - "key" of structure, "value"
// pub struct StructuresUpper(pub String, pub Vec<StructuresMidd>);
#[derive(Clone, Debug)]
/// StructuresMidd(key, value) - "key" of structure field, "value"
pub struct StructuresMidd(pub String, pub Vec<String>);

#[derive(Clone, Debug)]
/// SGlobal(key, value)
pub struct SGlobal(pub String, pub String);

#[derive(Clone, Debug)]
pub struct SchemaService {
    pub v: String,
    pub global: Vec<SGlobal>,
    // v2?: "count": {"type":"","description":[""],"deprecated":false}
    /// "count": \["TYPE", "description", "description more"...]
    ///
    /// ":description": \["description", "description more"...]
    ///
    pub structures: HashMap<String, Vec<StructuresMidd>>,
    // pub structures: Vec<StructuresUpper>,
    // TODO:
    // pub functions:
}

pub struct Service {}

impl Service {
    pub fn do_diagnostic(document: Document, _schema: Option<SchemaService>) -> Vec<Diagnostic> {
        let source = document.get_text();
        let mut diagnostics: Vec<Diagnostic> = Vec::new();
        let ast_r = Service::diagnostic_parser(source, &mut diagnostics);

        if ast_r.is_none() {
            return diagnostics;
        }

        return diagnostics;
    }

    fn diagnostic_parser(
        source: String,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<ParserResults> {
        let mut lexr = Lexer::from_source(&source);
        let lexr_res = lexr.scan_tokens();
        if lexr_res.is_err() {
            let err_msg = lexr_res.err().unwrap();
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        character: lexr.position.line as u32,
                        line: lexr.position.column as u32,
                    },
                    end: Position {
                        character: (lexr.position.line + 1) as u32,
                        line: lexr.position.column as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: err_msg,
                ..Default::default()
            });
            return None;
        }

        let mut parser = Parser::from_lexer(lexr);
        let parse_results = parser.parse();
        for err in &parse_results.errors {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        character: err.start.line as u32,
                        line: err.start.column as u32,
                    },
                    end: Position {
                        character: err.end.line as u32,
                        line: err.end.column as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: err.message.clone(),
                ..Default::default()
            });
        }

        return Some(parse_results);
    }

    fn _diagnostic_types() {}

    // pub fn do_hover() {}

    pub fn do_autocomplete(
        mut document: Document,
        position: Position,
        schema: Option<SchemaService>,
    ) -> CompletionList {
        let source = document.get_text();
        let ast = Service::parse_ast(&source);
        if ast.is_none() {
            return CompletionList {
                is_incomplete: false,
                items: vec![],
            };
        }

        let offset = document.offset_at(jtl_parser::lex::PPosition {
            column: position.line as usize,
            line: position.character as usize,
        });

        let ast_source = ast.unwrap();
        let node_res = get_node_at_offset(&mut document, offset, &ast_source);

        // println!("{:?}", node_res);
        // println!("{:?}", offset);
        // println!("{:#?}", ast_source.body);
        if node_res.is_none() {
            return CompletionList {
                is_incomplete: false,
                items: vec![],
            };
        }
        let node: parser_get_node_at::Node = node_res.unwrap();

        match node {
            Node::Expression => {
                if schema.is_some() {
                    let mut items: Vec<CompletionItem> = Vec::new();
                    for SGlobal(key, _value) in schema.unwrap().global {
                        // TODO: set kind depending on 'value'
                        items.push(CompletionItem {
                            label: key,
                            kind: Some(CompletionItemKind::TEXT),
                            ..Default::default()
                        })
                    }

                    CompletionList {
                        is_incomplete: false,
                        items,
                    }
                } else {
                    CompletionList {
                        is_incomplete: false,
                        items: vec![],
                    }
                }
            }
            // {guild.name}
            Node::Property(property) => {
                if schema.is_none() {
                    return CompletionList {
                        is_incomplete: false,
                        items: vec![],
                    };
                };

                let schema_safe = schema.unwrap();
                let content_start = position.character - (property.start.line as u32) + 1;
                let content_full = property.value.join(".");
                // println!("content_full: {:?}", content_full);
                // println!("content_start: {:?}", content_start);
                let (content_complete, _): (&str, &str) = {
                    if content_start as usize >= content_full.len() {
                        (content_full.as_str(), "")
                    } else {
                        content_full.split_at(content_start as usize)
                    }
                };

                // println!("content_complete: {:?}", content_complete);
                let char_at =
                    source.chars().collect::<Vec<_>>()[position.character as usize].to_string();
                println!("char_at: {:?}", char_at);

                // If this is true, then it should complete the final ident or None (no regex)
                // Ex: "{guild.}" char position is on the '.'
                let send_final_struct_full = char_at == ".";

                let mut index: usize = 0;
                let mut base_struct: Option<&Vec<StructuresMidd>> = None;
                let content_split: std::str::Split<'_, &str> = content_complete.split(".");
                let content_split_len = &content_split.clone().collect::<Vec<&str>>().len();
                for c in content_split {
                    index += 1;
                    if &index == content_split_len {
                        if send_final_struct_full {
                            let mut items: Vec<CompletionItem> = Vec::new();
                            for StructuresMidd(key, _value) in base_struct.unwrap() {
                                // TODO: set kind depending on 'value'
                                items.push(CompletionItem {
                                    label: key.clone(),
                                    kind: Some(CompletionItemKind::TEXT),
                                    ..Default::default()
                                })
                            }

                            return CompletionList {
                                is_incomplete: false,
                                items,
                            };
                        } else {
                            println!("base_struct: {:?}", &base_struct);
                            // TODO: Regex search
                        }
                        continue;
                    }
                    // terverse though the schema... >

                    // Top level terversal. Aka global, th rest should use structures...
                    if index == 1 {
                        let key: Option<String> = {
                            let mut r = None;
                            for SGlobal(key, value) in &schema_safe.global {
                                if key == c {
                                    r = Some(value.clone());
                                    break;
                                }
                            }
                            r
                        };

                        if key.is_some() {
                            let key_safe = key.unwrap();
                            if key_safe.starts_with("#") {
                                let structure_res = &schema_safe.structures.get(&key_safe[1..]);
                                if structure_res.is_none() {
                                    return CompletionList {
                                        is_incomplete: false,
                                        items: vec![],
                                    };
                                }
                                let structure = structure_res.unwrap();
                                base_struct = Some(structure);
                            }
                        }
                        continue;
                    }

                    println!("If you've reached this message we currently doen't supported deeply nested object...\nFeel free to help - https://github.com/Wilbert-mad/jtl");
                }

                return CompletionList {
                    is_incomplete: false,
                    items: vec![],
                };
            }
            Node::Text => CompletionList {
                is_incomplete: false,
                items: vec![],
            },
            // Node::Argument(_) => {} // TODO:
        }
    }

    fn parse_ast(source: &String) -> Option<Source> {
        let mut lexr = Lexer::from_source(&source);
        let lexr_res = lexr.scan_tokens();
        if lexr_res.is_err() {
            return None;
        }

        let mut parser = Parser::from_lexer(lexr);
        Some(parser.parse().ast)
    }
}

// fn vec_find<T: std::cmp::PartialEq>(v: &Vec<T>, w: &T) -> Option<&'static T> {
//     for item in v {
//         if item == w {
//             return Some(&w);
//         }
//     }
//     None
// }

#[cfg(test)]
mod tests {
    use super::*;

    fn text_document_prop(content: String) -> Document {
        Document::new("//master".to_string(), "jtl".to_string(), 1, content)
    }

    #[test]
    fn diagnostic_test() {
        let mut structures = HashMap::new();
        let mut guild_struct = Vec::new();
        guild_struct.push(StructuresMidd(
            "name".to_string(),
            vec!["String".to_string()],
        ));

        structures.insert("Guild".to_string(), guild_struct);

        let schema = SchemaService {
            v: "1.0.0".to_string(),
            global: vec![SGlobal("guild".to_string(), "#Guild".to_string())],
            structures,
            // structures: vec![StructuresUpper("Guild".to_string(), guild_struct)],
        };

        let source = "start
        {}"
        .to_string();
        // let source = "sound{guild.n}".to_string();
        // println!(
        //     "{:?}",
        //     Service::do_diagnostic(text_document_prop(source.clone()), Some(schema.clone()))
        // );
        println!(
            "{:#?}",
            Service::do_autocomplete(
                text_document_prop(source.clone()),
                Position {
                    line: 1,
                    character: 9 // |
                                 // character: 8 // gui|
                                 // character: 11 // guild.|
                                 // character: 12 // guild.n|
                                 // character: 13 // guild.ne|
                },
                Some(schema)
            )
        );
    }
}
