use proc_macro2::{Ident, Span};
use quote::quote;

use snax::{
    SnaxItem,
    SnaxTag,
};

#[test]
fn empty_div() {
    let input = quote!(<div></div>);
    let output = snax::parse(input).unwrap();

    assert_eq!(output, SnaxItem::Tag(SnaxTag {
        name: Ident::new("div", Span::call_site()),
        attributes: Default::default(),
        children: Default::default(),
    }));
}