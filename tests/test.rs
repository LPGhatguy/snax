use quote::quote;

use snax::{
    SnaxItem,
};

#[test]
fn empty_div() {
    let input = quote!(<div></div>);
    let output = snax::parse(input).unwrap();

    let tag = match output {
        SnaxItem::Tag(tag) => tag,
        _ => panic!("Expected SnaxItem::Tag, got {:?}", output),
    };

    assert_eq!(tag.name.to_string(), "div");
    assert_eq!(tag.attributes.len(), 0);
    assert_eq!(tag.children.len(), 0);
}