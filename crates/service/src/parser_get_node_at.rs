// This is a port of marko's lsp get-node-at-offset
// MIT Licenced -https://github.com/marko-js/language-server/blob/main/packages/language-tools/src/util/get-node-at-offset.ts

use jtl_parser::parser::{PValue, Property, Source, Stat};

use crate::document::Document;

#[derive(Debug)]
pub enum Node<'a> {
    /// If the node is of type text
    Text,
    /// If the node is empty('{}')
    Expression,
    /// If the node is within the property section
    Property(&'a Property),
    // Argumment(),
}

pub fn get_node_at_offset<'a>(
    document: &'a mut Document,
    offset: u32,
    program: &'a Source,
) -> Option<Node<'a>> {
    let child_node = child_at_offset(document, offset, &program.body);
    // println!("YYYY {:?}", &child_node);
    // println!("YYYY {:?} {:?}", offset, &program.body);
    if child_node.is_some() {
        return visit_child_node(document, offset, &child_node.unwrap());
    }
    None
}

fn visit_child_node<'c>(
    document: &'c mut &Document,
    offset: u32,
    node: &'c Stat,
) -> Option<Node<'c>> {
    match node {
        Stat::Text {
            _type: _,
            value: _,
            start: _,
            end: _,
        } => Some(Node::Text),
        Stat::Tag {
            _type: _,
            start: _,
            end: _,
            value,
        } => {
            if value.property.is_none() {
                return Some(Node::Expression);
            }
            let PValue::Property(property) = value.property.as_ref().unwrap()
            else {todo!()};

            // Aka past the property and likey arguments
            if offset > (document.offset_at(property.end.clone())) {
                return None;
            }

            Some(Node::Property(property))
        }
    }
}

fn child_at_offset<'b>(
    document: &'b mut &Document,
    offset: u32,
    children: &'b Vec<Stat>,
) -> Option<&'b Stat> {
    let mut max = children.len() as i32 - 1i32;
    if max == -1 {
        return None;
    }

    let mut min = 0i32;
    while min < max {
        let mid = (1 + min + max) >> 1;

        let child = children.get(mid as usize).unwrap();
        let start = match child {
            Stat::Tag {
                _type,
                start,
                end: _,
                value: _,
            } => start,
            Stat::Text {
                _type,
                start,
                end: _,
                value: _,
            } => start,
        };

        if (document.offset_at(start.clone()) as u32) < offset {
            min = mid;
        } else {
            max = mid - 1
        }
    }

    let child = children.get(min as usize).unwrap();
    let (start, end) = match child {
        Stat::Tag {
            _type,
            start,
            end,
            value: _,
        } => (start, end),
        Stat::Text {
            _type,
            start,
            end,
            value: _,
        } => (start, end),
    };

    // println!(
    //     "{:?} -- {:?}",
    //     document.offset_at(start.clone()),
    //     document.offset_at(end.clone())
    // );
    // // println!("{:?} -- {:?}", start.to_pointer(), end.to_pointer());
    // // println!("{:?}", child);
    if offset > document.offset_at(start.clone()) && offset <= document.offset_at(end.clone()) {
        Some(&child)
    } else {
        None
    }
}
