use serde::ser::SerializeMap;
use serde::ser::SerializeSeq;
use serde::Serialize;
use serde::Serializer;

use std::io::Read;

use itertools::Itertools;

use pulldown_cmark::{
    Event::{End, Start, Text},
    HeadingLevel, Options, Parser, Tag,
};

/*
 * todo:
 * if header has no children... null?
 */

fn main() {
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s).unwrap();
    println!("{}", serde_json::to_string(&mnj(&s)).unwrap());
}

#[derive(Debug, PartialEq)]
enum Node {
    Header(Vec<(String, Vec<Node>)>),
    Items(Vec<Node>),
    Leaf(String),
}

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Node::Header(header) => {
                let mut map = serializer.serialize_map(Some(header.len()))?;
                for (k, v) in header {
                    if v.len() == 1 {
                        map.serialize_entry(k, &v[0])?;
                    } else {
                        map.serialize_entry(k, v)?;
                    }
                }
                map.end()
            }
            Node::Items(items) => {
                let mut seq = serializer.serialize_seq(Some(items.len()))?;
                for e in items {
                    seq.serialize_element(e)?;
                }
                seq.end()
            }
            Node::Leaf(s) => serializer.serialize_str(s),
        }
    }
}

fn compact_nodes(mut nodes: Vec<Node>) -> Node {
    if nodes.len() == 1 {
        return nodes.pop().unwrap();
    }
    return Node::Items(nodes);
}

fn _go(parser: &mut Parser) -> Node {
    let mut ret = Vec::<Node>::new();
    let mut items = Vec::<Node>::new();
    let mut depth = Vec::<(HeadingLevel, Vec<(String, Vec<Node>)>)>::new();

    while let Some(next) = parser.next() {
        match next {
            Start(Tag::Heading(level, None, classes)) => {
                assert!(classes.is_empty(), "todo: what are classes?");

                let header = match _go(parser) {
                    Node::Leaf(s) => s,
                    _ => todo!(),
                };

                let curr = depth.pop();
                match curr {
                    Some(curr) => {
                        let (curr_level, mut curr_headers) = curr;

                        let (last_header_title, mut last_header_nodes) =
                            curr_headers.pop().unwrap();
                        last_header_nodes.append(&mut items);
                        curr_headers.push((last_header_title, last_header_nodes));

                        // open a sub-header
                        if level > curr_level {
                            depth.push((curr_level, curr_headers));
                            depth.push((level, vec![(header, vec![])]));
                            continue;
                        }

                        // close current header, open sibling header
                        if level == curr_level {
                            curr_headers.push((header, vec![]));
                            depth.push((curr_level, curr_headers));
                            continue;
                        }

                        // close current header, open parent header
                        ret.push(Node::Header(curr_headers));
                        depth.push((level, vec![(header, vec![])]));
                    }

                    // no headers open
                    None => {
                        depth.push((level, vec![(header, vec![])]));
                    }
                }
            }

            Start(Tag::Paragraph) => {
                let item = _go(parser);
                items.push(item);
            }

            Start(Tag::List(_)) => {
                let node = _go(parser);
                let node = match node {
                    Node::Items(items) => Node::Items(items) ,
                    Node::Leaf(text) => Node::Items(vec![Node::Leaf(text)]),
                    _ => todo!(),
                };
                items.push(node);
            }

            Start(Tag::Item) => {
                let node = _go(parser);

                // todo: should put this behind a cli flag
                let node = match node {
                    Node::Items(mut subitems) => {
                        if let Some((Node::Leaf(_), Node::Items(_))) =
                            subitems.iter().collect_tuple()
                        {
                            if let (Node::Leaf(key), Node::Items(values)) =
                                subitems.drain(..).collect_tuple().unwrap()
                            {
                                Node::Header(vec![(key.to_string(), values)])
                            } else {
                                unimplemented!()
                            }
                        } else {
                            Node::Items(subitems)
                        }
                    }

                    Node::Leaf(text) => Node::Leaf(text),

                    todo => {
                        panic!("todo: {:?}", todo);
                    }
                };

                items.push(node);
            }

            Text(text) => items.push(Node::Leaf(text.to_string())),

            End(_) => break,

            todo => {
                panic!("todo: {:?}", todo);
            }
        }
    }

    // close remaining open headers
    while let Some((_, mut curr_headers)) = depth.pop() {
        let (last_header_title, mut last_header_nodes) = curr_headers.pop().unwrap();
        last_header_nodes.append(&mut items);
        curr_headers.push((last_header_title, last_header_nodes));
        items.push(Node::Header(curr_headers));
    }

    ret.append(&mut items);
    return compact_nodes(ret);
}

fn mnj(markdown: &str) -> Node {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TASKLISTS);
    let mut parser = Parser::new_ext(markdown, options);
    let node = _go(&mut parser);
    return node;
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    // use pretty_assertions::assert_eq;

    #[test]
    fn plain_text() {
        let got = mnj(indoc! {"
        plain text
        "});
        assert_eq!(serde_json::to_string(&got).unwrap(), r#""plain text""#);
    }

    #[test]
    fn header_to_plain_text() {
        let got = mnj(indoc! {"
        # Todo
        Foo
        "});
        assert_eq!(serde_json::to_string(&got).unwrap(), r#"{"Todo":"Foo"}"#);
    }

    #[test]
    fn list() {
        let got = mnj(indoc! {"
        - one
        - two
        - three
        "});
        assert_eq!(
            serde_json::to_string(&got).unwrap(),
            r#"["one","two","three"]"#
        );
    }

    #[test]
    fn list_with_one_item() {
        let got = mnj(indoc! {"
        - one
        "});
        assert_eq!(
            serde_json::to_string(&got).unwrap(),
            r#"["one"]"#
        );
    }

    #[test]
    fn nested_list() {
        let got = mnj(indoc! {"
        - one
            - one.1
            - one.2
        - two
        - three
            - three.1
        "});
        // todo: shouldn't three.1 be a list?
        assert_eq!(
            serde_json::to_string(&got).unwrap(),
            r#"[{"one":["one.1","one.2"]},"two",{"three":"three.1"}]"#
        );
    }

    #[test]
    fn sibling_headers() {
        let got = mnj(indoc! {"
        # Todo
        do it
        # More
        even
        "});
        assert_eq!(
            serde_json::to_string(&got).unwrap(),
            r#"{"Todo":"do it","More":"even"}"#
        );
    }

    #[test]
    fn nested_headers() {
        let got = mnj(indoc! {"
        # Todo
        do it
        ## More
        even
        "});
        assert_eq!(
            serde_json::to_string(&got).unwrap(),
            r#"{"Todo":["do it",{"More":"even"}]}"#
        );
    }
}
