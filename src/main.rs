use std::collections::{HashMap, VecDeque};
 use std::io::Read;

use serde::Serialize;

use markdown::Block::{Header, Paragraph, UnorderedList};
use markdown::{Block, ListItem, Span};

fn main() {
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s).unwrap();
    println!("{}", serde_json::to_string(&m2j(&s)).unwrap());
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Node {
    Header(HashMap<String, Node>),
    Items(Vec<Node>),
    Leaf(String),
}

fn m2j(s: &str) -> Node {
    let blocks = markdown::tokenize(s);
    let mut blocks = VecDeque::from(blocks);
    let node = _blocks(&mut blocks);

    if blocks.is_empty() {
        return node;
    }

    let mut items = vec![node];
    while !blocks.is_empty() {
        items.push(_blocks(&mut blocks));
    }
    return Node::Items(items);
}

fn spans_to_markdown(spans: &Vec<Span>) -> String {
    return markdown::generate_markdown(vec![Paragraph(spans.to_vec())]);
}

fn _headers(
    map: &mut HashMap<String, Node>,
    blocks: &mut VecDeque<Block>,
    spans: Vec<Span>,
    size: &usize,
) {
    map.insert(spans_to_markdown(&spans), _blocks(blocks));

    if let Some(Header(_, next_size)) = blocks.front() {
        if next_size >= size {
            match blocks.pop_front() {
                Some(Header(next_spans, next_size)) => {
                    _headers(map, blocks, next_spans.to_vec(), &next_size);
                }
                _ => unimplemented!(),
            }
        }
    }
}

fn _blocks(blocks: &mut VecDeque<Block>) -> Node {
    match &blocks.pop_front().unwrap() {
        Header(spans, size) => {
            let mut map = HashMap::new();
            _headers(&mut map, blocks, spans.to_vec(), size);
            Node::Header(map)
        }

        UnorderedList(items) => {
            let items = items.iter().map(|item| match &item {
                ListItem::Simple(spans) => Node::Leaf(spans_to_markdown(&spans)),
                ListItem::Paragraph(blocks) => {
                    let mut blocks = VecDeque::from(blocks.to_owned());
                    match blocks.pop_front().unwrap() {
                        Block::Paragraph(spans) => {
                            let header = spans_to_markdown(&spans);
                            let node = _blocks(&mut blocks);
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


#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_m2j_complex() {
        let res = m2j(indoc! {"
        # Todo

        ## Work
        - one
            - one.1
        - two
        - three

        ## Home
        - order shelving

        # SaaS
        - [ ] markdown to json cli
        "}
        .into());

        let got = serde_json::to_string_pretty(&res).unwrap();
        //println!("{}", got);

        assert_eq!(
            got,
            indoc! {r#"
            {
              "SaaS": [
                "[ ] markdown to json cli"
              ],
              "Todo": {
                "Work": [
                  {
                    "one": [
                      "one.1"
                    ]
                  },
                  "two",
                  "three"
                ],
                "Home": [
                  "order shelving"
                ]
              }
            }"#}
        );
    }

    #[test]
    fn test_m2j_header_without_children() {
        let res = m2j(indoc! {"
        # Todo

        ## Work
        "}
        .into());

        let got = serde_json::to_string_pretty(&res).unwrap();
        //println!("{}", got);

        assert_eq!(
            got,
            indoc! {r#"
            {
              "Todo": "Work"
              }
            "#}
        );
    }
}
