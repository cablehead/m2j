use std::collections::HashMap;

use serde::Serialize;

use markdown::Block::{Header, Paragraph, UnorderedList};
use markdown::{Block, ListItem, Span};

fn main() {
    println!("Hello, world!");
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Node {
    Header(HashMap<String, Node>),
    Items(Vec<Node>),
    Leaf(String),
}

fn spans_to_markdown(spans: &Vec<Span>) -> String {
    return markdown::generate_markdown(vec![Paragraph(spans.to_vec())]);
}

fn sub_m2j(mut blocks: std::slice::Iter<Block>) -> Node {
    match &blocks.next().unwrap() {
        Header(spans, _size) => {
            let header = spans_to_markdown(&spans);
            let node = sub_m2j(blocks);
            let map = HashMap::from([(header, node)]);
            Node::Header(map)
        }

        UnorderedList(items) => {
            let items = items.iter().map(|item| match &item {
                ListItem::Simple(spans) => Node::Leaf(spans_to_markdown(&spans)),
                ListItem::Paragraph(blocks) => {
                    let mut blocks = blocks.iter();
                    match blocks.next().unwrap() {
                        Block::Paragraph(spans) => {
                            let header = spans_to_markdown(&spans);
                            let node = sub_m2j(blocks);
                            let map = HashMap::from([(header, node)]);
                            Node::Header(map)
                        }
                        _ => todo!(),
                    }
                }
            });
            Node::Items(items.collect::<Vec<Node>>())
        }

        _ => todo!(),
    }
}

fn m2j(s: &str) -> Node {
    let blocks = markdown::tokenize(s);
    let blocks = blocks.iter();
    return sub_m2j(blocks);
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_m2j() {
        let j = m2j(indoc! {"
    # Todo
    - one
        - one.1
    - two
    - three"}
        .into());
        println!("{}", serde_json::to_string(&j).unwrap());
    }
}
