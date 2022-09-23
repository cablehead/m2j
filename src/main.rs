use std::collections::HashMap;

use markdown::Block::{Header, Paragraph, UnorderedList};
use markdown::{ListItem, Span};

fn main() {
    println!("Hello, world!");
}

fn spans_to_markdown(spans: &Vec<Span>) -> String {
    return markdown::generate_markdown(vec![Paragraph(spans.to_vec())]);
}

fn m2j(s: &str) {
    let tokens = markdown::tokenize(s);
    // let j = serde_json::to_string(&tokens).unwrap();
    let mut d = HashMap::new();

    for token in tokens {
        match &token {
            Header(spans, _size) => {
                let header = spans_to_markdown(&spans);
                d.insert(header, "foo");
            }

            UnorderedList(items) => {
                for item in items {
                    match &item {
                        ListItem::Simple(spans) => println!("{}", spans_to_markdown(&spans)),
                        _ => todo!(),
                    }
                }
            }

            _ => todo!(),
        }
    }
    println!("{}", serde_json::to_string(&d).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_m2j() {
        m2j(indoc! {"
    # Todo
    - one
    - two
    - three"}
        .into());
    }
}
