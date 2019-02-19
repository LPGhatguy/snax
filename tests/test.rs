use proc_macro2::{Ident, Span};
use quote::quote;

use snax::{
    SnaxItem,
    SnaxTag,
    SnaxSelfClosingTag,
    SnaxAttribute,
    SnaxFragment,
};

/// Like quote!, but returns a single TokenTree instead
macro_rules! quote_one {
    ($($value: tt)*) => {
        quote!($($value)*).into_iter().next().unwrap()
    };
}

#[test]
fn just_string() {
    let input = quote!("hello");
    let output = snax::parse(input).unwrap();

    let expected = SnaxItem::Content(quote_one!("hello"));
    assert_eq!(output, expected);
}

#[test]
fn just_number() {
    let input = quote!(5);
    let output = snax::parse(input).unwrap();

    let expected = SnaxItem::Content(quote_one!(5));
    assert_eq!(output, expected);
}

#[test]
fn empty_div() {
    let input = quote!(<div></div>);
    let output = snax::parse(input).unwrap();

    let expected = SnaxItem::Tag(SnaxTag {
        name: Ident::new("div", Span::call_site()),
        attributes: Default::default(),
        children: Default::default(),
    });

    assert_eq!(output, expected);
}

#[test]
fn self_closing_div() {
    let input = quote!(<div />);
    let output = snax::parse(input).unwrap();

    let expected = SnaxItem::SelfClosingTag(SnaxSelfClosingTag {
        name: Ident::new("div", Span::call_site()),
        attributes: Default::default(),
    });

    assert_eq!(output, expected);
}

#[test]
fn empty_div_comment() {
    let input = quote!(<div>/* Hello, world! */</div>);
    let output = snax::parse(input).unwrap();

    let expected = SnaxItem::Tag(SnaxTag {
        name: Ident::new("div", Span::call_site()),
        attributes: Default::default(),
        children: Default::default(),
    });

    assert_eq!(output, expected);
}

#[test]
fn empty_fragment() {
    let input = quote!(<> </>);
    let output = snax::parse(input).unwrap();

    let expected = SnaxItem::Fragment(SnaxFragment {
        children: Default::default(),
    });

    assert_eq!(output, expected);
}

#[test]
fn empty_with_literal_attributes() {
    let input = quote!(<div foo="bar" baz="qux"></div>);
    let output = snax::parse(input).unwrap();

    let expected = SnaxItem::Tag(SnaxTag {
        name: Ident::new("div", Span::call_site()),
        attributes: vec![
            SnaxAttribute::Simple {
                name: Ident::new("foo", Span::call_site()),
                value: quote_one!("bar"),
            },
            SnaxAttribute::Simple {
                name: Ident::new("baz", Span::call_site()),
                value: quote_one!("qux"),
            },
        ],
        children: Default::default(),
    });

    assert_eq!(output, expected);
}

#[test]
fn empty_with_block_attribute() {
    let input = quote!(<label sum={ 5 + 5 }></label>);
    let output = snax::parse(input).unwrap();

    let expected = SnaxItem::Tag(SnaxTag {
        name: Ident::new("label", Span::call_site()),
        attributes: vec![
            SnaxAttribute::Simple {
                name: Ident::new("sum", Span::call_site()),
                value: quote_one!({ 5 + 5 }),
            },
        ],
        children: Default::default(),
    });

    assert_eq!(output, expected);
}

#[test]
fn self_closing_with_literal_attributes() {
    let input = quote!(<div foo="bar" baz="qux" />);
    let output = snax::parse(input).unwrap();

    let expected = SnaxItem::SelfClosingTag(SnaxSelfClosingTag {
        name: Ident::new("div", Span::call_site()),
        attributes: vec![
            SnaxAttribute::Simple {
                name: Ident::new("foo", Span::call_site()),
                value: quote_one!("bar"),
            },
            SnaxAttribute::Simple {
                name: Ident::new("baz", Span::call_site()),
                value: quote_one!("qux"),
            },
        ],
    });

    assert_eq!(output, expected);
}

#[test]
fn self_closing_with_block_attribute() {
    let input = quote!(<label sum={ 5 + 5 } />);
    let output = snax::parse(input).unwrap();

    let expected = SnaxItem::SelfClosingTag(SnaxSelfClosingTag {
        name: Ident::new("label", Span::call_site()),
        attributes: vec![
            SnaxAttribute::Simple {
                name: Ident::new("sum", Span::call_site()),
                value: quote_one!({ 5 + 5 }),
            },
        ],
    });

    assert_eq!(output, expected);
}

#[test]
fn nested_tags() {
    let input = quote!(
        <div>
            <span></span>
        </div>
    );
    let output = snax::parse(input).unwrap();

    let expected = SnaxItem::Tag(SnaxTag {
        name: Ident::new("div", Span::call_site()),
        attributes: Default::default(),
        children: vec![
            SnaxItem::Tag(SnaxTag {
                name: Ident::new("span", Span::call_site()),
                attributes: Default::default(),
                children: Default::default(),
            }),
        ],
    });

    assert_eq!(output, expected);
}

#[test]
fn adjacent_tags() {
    let input = quote!(
        <div>
            <span></span>
            <div></div>
        </div>
    );
    let output = snax::parse(input).unwrap();

    let expected = SnaxItem::Tag(SnaxTag {
        name: Ident::new("div", Span::call_site()),
        attributes: Default::default(),
        children: vec![
            SnaxItem::Tag(SnaxTag {
                name: Ident::new("span", Span::call_site()),
                attributes: Default::default(),
                children: Default::default(),
            }),
            SnaxItem::Tag(SnaxTag {
                name: Ident::new("div", Span::call_site()),
                attributes: Default::default(),
                children: Default::default(),
            }),
        ],
    });

    assert_eq!(output, expected);
}

#[test]
fn adjacent_tags_in_fragment() {
    let input = quote!(
        <>
            <span></span>
            <div></div>
        </>
    );
    let output = snax::parse(input).unwrap();

    let expected = SnaxItem::Fragment(SnaxFragment {
        children: vec![
            SnaxItem::Tag(SnaxTag {
                name: Ident::new("span", Span::call_site()),
                attributes: Default::default(),
                children: Default::default(),
            }),
            SnaxItem::Tag(SnaxTag {
                name: Ident::new("div", Span::call_site()),
                attributes: Default::default(),
                children: Default::default(),
            }),
        ],
    });

    assert_eq!(output, expected);
}

#[test]
fn nested_tags_self_closing() {
    let input = quote!(
        <div>
            <span />
        </div>
    );
    let output = snax::parse(input).unwrap();

    let expected = SnaxItem::Tag(SnaxTag {
        name: Ident::new("div", Span::call_site()),
        attributes: Default::default(),
        children: vec![
            SnaxItem::SelfClosingTag(SnaxSelfClosingTag {
                name: Ident::new("span", Span::call_site()),
                attributes: Default::default(),
            }),
        ],
    });

    assert_eq!(output, expected);
}

#[test]
fn content_literal_string() {
    let input = quote!(
        <div>
            "Hello, world!"
        </div>
    );
    let output = snax::parse(input).unwrap();

    let expected = SnaxItem::Tag(SnaxTag {
        name: Ident::new("div", Span::call_site()),
        attributes: Default::default(),
        children: vec![
            SnaxItem::Content(quote_one!("Hello, world!")),
        ],
    });

    assert_eq!(output, expected);
}

#[test]
fn content_literal_number() {
    let input = quote!(
        <div>
            5
        </div>
    );
    let output = snax::parse(input).unwrap();

    let expected = SnaxItem::Tag(SnaxTag {
        name: Ident::new("div", Span::call_site()),
        attributes: Default::default(),
        children: vec![
            SnaxItem::Content(quote_one!(5)),
        ],
    });

    assert_eq!(output, expected);
}

#[test]
fn content_block() {
    let input = quote!(
        <div>
            { format!("{} + {} = {}", 1, 2, 3) }
        </div>
    );
    let output = snax::parse(input).unwrap();

    let expected = SnaxItem::Tag(SnaxTag {
        name: Ident::new("div", Span::call_site()),
        attributes: Default::default(),
        children: vec![
            SnaxItem::Content(quote_one!({ format!("{} + {} = {}", 1, 2, 3) })),
        ],
    });

    assert_eq!(output, expected);
}