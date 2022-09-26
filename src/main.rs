use serde::Serialize;

use markdown::Block::{Header, Paragraph, UnorderedList};
use markdown::{Block, ListItem, Span};

fn main() {
    println!("Hello, world!");
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Node {
    Leaf(String),
    Tree(String, Vec<Node>),
}

fn spans_to_markdown(spans: &Vec<Span>) -> String {
    return markdown::generate_markdown(vec![Paragraph(spans.to_vec())]);
}

fn sub_m2j(mut tokens: std::slice::Iter<Block>) -> Vec<Node> {
    while let Some(token) = tokens.next() {
        match &token {
            Header(spans, _size) => {
                let header = spans_to_markdown(&spans);
                let children = sub_m2j(tokens);
                return vec![Node::Tree(header, children)];
            }

            UnorderedList(items) => {
                let items = items.iter().map(|item| match &item {
                    ListItem::Simple(spans) => Node::Leaf(spans_to_markdown(&spans)),
                    _ => todo!(),
                });
                return items.collect::<Vec<Node>>();
            }

            _ => todo!(),
        }
    }
    return vec![];
}

fn m2j(s: &str) -> Vec<Node> {
    let tokens = markdown::tokenize(s);
    let tokens = tokens.iter();

    return sub_m2j(tokens);

    /*

    // let j = serde_json::to_string(&tokens).unwrap();
    let mut d = HashMap::new();


    */
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
    - two
    - three"}
        .into());
    println!("{}", serde_json::to_string(&j).unwrap());
    }
}
