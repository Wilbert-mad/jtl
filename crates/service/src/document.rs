// A semi-port of vscode textDocument. only importing what I'll be using...
// https://github.com/microsoft/vscode-languageserver-node/blob/main/textDocument/src/main.ts
// MIT License - https://github.com/microsoft/vscode-languageserver-node/blob/main/License.txt

use jtl_parser::lex::PPosition;
use std::cmp;

pub struct Document {
    pub uri: String,
    pub langauge_id: String,
    pub version: u32,
    content: String,
    line_offsets: Option<Vec<u32>>,
}

impl Document {
    pub fn new(uri: String, langauge_id: String, version: u32, content: String) -> Self {
        Document {
            uri,
            langauge_id,
            version,
            content,
            line_offsets: None,
        }
    }

    fn get_line_offsets(&mut self) -> &Vec<u32> {
        if self.line_offsets == None {
            self.line_offsets = Some(compute_line_offsets(&self.content, true, None));
        }

        self.line_offsets.as_ref().unwrap()
    }

    pub fn get_text(&self) -> String {
        self.content.clone()
    }
    pub fn position_at(&self) {}

    pub fn offset_at(&mut self, position: PPosition) -> u32 {
        let line_offsets = self.get_line_offsets();
        if position.column >= line_offsets.len() {
            return self.content.len() as u32;
        } else if position.column < 0 {
            return 0;
        }

        let line_offset = line_offsets[position.column];
        let next_line_offset = if (position.column + 1) < line_offsets.len() {
            line_offsets[position.column + 1]
        } else {
            self.content.len() as u32
        };
        cmp::max(
            cmp::min(line_offset + (position.line as u32), next_line_offset),
            line_offset,
        )
    }
}

static LINE_FEED: u32 = 10; // "\n"
static CARRIAGE_RETURN: u32 = 13; // "\r"

fn compute_line_offsets(
    text: &String,
    is_at_line_start: bool,
    text_offset: Option<u32>,
) -> Vec<u32> {
    let mut results = if is_at_line_start {
        vec![text_offset.unwrap_or(0)]
    } else {
        Vec::new()
    };

    let chars = text.chars().collect::<Vec<_>>();

    let mut skip_next = false;
    // println!("--- {:?} ---", text);
    for i in 0..text.len() {
        if skip_next {
            skip_next = false;
            continue;
        }

        let ch = chars[i] as u32;
        if ch == LINE_FEED || ch == CARRIAGE_RETURN {
            if ch == CARRIAGE_RETURN && (i + 1) < text.len() && u32::from(chars[i + 1]) == LINE_FEED
            {
                skip_next = true;
            }
            results.push(text_offset.unwrap_or(0) + (i as u32) + 1)
        }
    }

    results
}
