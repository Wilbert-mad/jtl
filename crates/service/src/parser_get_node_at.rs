// This is a port of marko's lsp get-node-at-offset
// MIT Licenced -https://github.com/marko-js/language-server/blob/main/packages/language-tools/src/util/get-node-at-offset.ts

use jtl_parser::parser::{PValue, Property, Source, Stat};

use crate::document::Document;

#[derive(Debug)]
pub enum Node {
    /// If the node is of type text
    Text,
    /// If the node is empty('{}')
    Expression,
    /// If the node is within the property section
    Property(Property),
    // Argumment(),
}

pub fn get_node_at_offset(document: &mut Document, offset: u32, program: &Source) -> Option<Node> {
    let child_node = child_at_offset(document, offset, &program.body);
    // println!("YYYY {:?}", &child_node);
    // println!("YYYY {:?} {:?}", offset, &program.body);
    if child_node.is_some() {
        return visit_child_node(document, offset, &child_node.unwrap());
    }
    None
}

fn visit_child_node(document: &mut Document, offset: u32, node: &Stat) -> Option<Node> {
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

            let property_owned = (*property).clone();
            Some(Node::Property(property_owned))
        }
    }
}

fn child_at_offset(document: &mut Document, offset: u32, children: &Vec<Stat>) -> Option<Stat> {
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

    let child = children[min as usize].clone();
    let (start, end) = match child.clone() {
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
    if offset > document.offset_at(start) && offset <= document.offset_at(end) {
        Some(child)
    } else {
        None
    }
}
