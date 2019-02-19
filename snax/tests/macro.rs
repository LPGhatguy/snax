use std::{
    collections::HashMap,
    borrow::{Borrow, Cow},
};

use maplit::hashmap;

use snax::{
    HtmlTag,
    HtmlSelfClosingTag,
    HtmlContent,
    Fragment,
    snax,
};

fn compare<'a, 'b, A, B>(a: A, b: B)
    where A: Borrow<HtmlContent<'a>>,
          B: Borrow<HtmlContent<'b>>,
{
    let a = a.borrow();
    let b = b.borrow();

    if a != b {
        panic!("HtmlContent not the same!\nLeft: {:#?}\n{}\n\nRight: {:#?}\n{}", a, a, b, b);
    }
}

#[test]
fn just_string() {
    let tag = snax!("hi");

    compare(tag, HtmlContent::from("hi"));
}

#[test]
fn composing_via_function() {
    fn my_component(value: &str) -> HtmlContent {
        snax!(
            <span class="hello">{ value }</span>
        )
    }

    // We want this to be a string to make sure the inferred lifetime for
    // my_component is NOT 'static.
    let cool_value = "hello".to_owned();

    let tag = snax!(
        <div>
            { my_component(&cool_value) }
        </div>
    );

    compare(tag, HtmlContent::Tag(HtmlTag {
        name: Cow::Borrowed("div"),
        attributes: HashMap::new(),
        children: vec![
            HtmlContent::Tag(HtmlTag {
                name: Cow::Borrowed("span"),
                attributes: hashmap! {
                    Cow::Borrowed("class") => Cow::Borrowed("hello"),
                },
                children: vec![
                    "hello".into(),
                ],
            })
        ],
    }))
}

#[test]
fn empty() {
    let tag = snax!(<div></div>);

    compare(tag, HtmlContent::Tag(HtmlTag {
        name: Cow::Borrowed("div"),
        attributes: HashMap::new(),
        children: Vec::new(),
    }));
}

#[test]
fn empty_comment() {
    let tag = snax!(
        <div>
            /* Hello, world! */
        </div>
    );

    compare(tag, HtmlContent::Tag(HtmlTag {
        name: Cow::Borrowed("div"),
        attributes: HashMap::new(),
        children: Vec::new(),
    }));
}

#[test]
fn self_closing() {
    let tag = snax!(<div />);

    compare(tag, HtmlContent::SelfClosingTag(HtmlSelfClosingTag {
        name: Cow::Borrowed("div"),
        attributes: HashMap::new(),
    }));
}

#[test]
fn empty_fragment() {
    let fragment = snax!(<> </>);

    compare(fragment, HtmlContent::Fragment(Fragment {
        children: Vec::new(),
    }));
}

#[test]
fn empty_with_attributes() {
    let tag = snax!(<div foo="bar" baz="qux"></div>);

    compare(tag, HtmlContent::Tag(HtmlTag {
        name: Cow::Borrowed("div"),
        attributes: hashmap! {
            Cow::Borrowed("foo") => Cow::Borrowed("bar"),
            Cow::Borrowed("baz") => Cow::Borrowed("qux"),
        },
        children: Vec::new(),
    }));
}

#[test]
fn empty_with_block_attribute() {
    let tag = snax!(<div foo={ (5 + 5).to_string() }></div>);

    compare(tag, HtmlContent::Tag(HtmlTag {
        name: Cow::Borrowed("div"),
        attributes: hashmap! {
            Cow::Borrowed("foo") => Cow::Borrowed("10"),
        },
        children: Vec::new(),
    }));
}

#[test]
fn self_closing_with_attribute() {
    let tag = snax!(<div foo="hello" />);

    compare(tag, HtmlContent::SelfClosingTag(HtmlSelfClosingTag {
        name: Cow::Borrowed("div"),
        attributes: hashmap! {
            Cow::Borrowed("foo") => Cow::Borrowed("hello"),
        },
    }));
}

#[test]
fn literal_string() {
    let tag = snax!(
        <span>
            "Hello, world!"
        </span>
    );

    compare(tag, HtmlContent::Tag(HtmlTag {
        name: Cow::Borrowed("span"),
        attributes: HashMap::new(),
        children: vec![
            "Hello, world!".into(),
        ],
    }));
}

#[test]
fn literal_string_fragment() {
    let fragment = snax!(
        <>
            "Hello!"
        </>
    );

    compare(fragment, HtmlContent::Fragment(Fragment {
        children: vec![
            "Hello!".into(),
        ],
    }));
}

#[test]
fn literal_multiple_string_fragment() {
    let fragment = snax!(
        <>
            "Hello, "
            "world!"
        </>
    );

    compare(fragment, HtmlContent::Fragment(Fragment {
        children: vec![
            "Hello, ".into(),
            "world!".into(),
        ],
    }));
}

#[test]
fn literal_block() {
    let tag = snax!(
        <span>
            { (5 + 5).to_string() }
        </span>
    );

    compare(tag, HtmlContent::Tag(HtmlTag {
        name: Cow::Borrowed("span"),
        attributes: HashMap::new(),
        children: vec![
            "10".into(),
        ],
    }));
}

#[test]
fn literal_block_fragment() {
    let tag = snax!(
        <span>
            { Fragment::new(["hello", "world"].iter()) }
        </span>
    );

    compare(tag, HtmlContent::Tag(HtmlTag {
        name: Cow::Borrowed("span"),
        attributes: HashMap::new(),
        children: vec![
            "hello".into(),
            "world".into(),
        ],
    }));
}

// https://github.com/LPGhatguy/snax/issues/7
//
// #[test]
// fn literal_block_iterator() {
//     let tag = snax!(
//         <span>
//             { ["hello", "world"].iter() }
//         </span>
//     );

//     compare(tag, HtmlContent::Tag(HtmlTag {
//         name: Cow::Borrowed("span"),
//         attributes: HashMap::new(),
//         children: vec![
//             "hello".into(),
//             "world".into(),
//         ],
//     }));
// }

// https://github.com/LPGhatguy/snax/issues/7
//
// #[test]
// fn literal_block_into_iterator() {
//     let tag = snax!(
//         <span>
//             { ["hello", "world"] }
//         </span>
//     );

//     compare(tag, HtmlContent::Tag(HtmlTag {
//         name: Cow::Borrowed("span"),
//         attributes: HashMap::new(),
//         children: vec![
//             "hello".into(),
//             "world".into(),
//         ],
//     }));
// }

#[test]
fn literal_block_content_fragments() {
    fn render_age(age: &u32) -> HtmlContent {
        snax!(
            <span>{ age.to_string() }</span>
        )
    }

    let tag = snax!(
        <div>
            { Fragment::new([32, 2, 114].iter().map(render_age)) }
        </div>
    );

    compare(tag, HtmlContent::Tag(HtmlTag {
        name: Cow::Borrowed("div"),
        attributes: HashMap::new(),
        children: vec![
            HtmlContent::Tag(HtmlTag {
                name: Cow::Borrowed("span"),
                attributes: HashMap::new(),
                children: vec![
                    "32".into(),
                ],
            }),
            HtmlContent::Tag(HtmlTag {
                name: Cow::Borrowed("span"),
                attributes: HashMap::new(),
                children: vec![
                    "2".into(),
                ],
            }),
            HtmlContent::Tag(HtmlTag {
                name: Cow::Borrowed("span"),
                attributes: HashMap::new(),
                children: vec![
                    "114".into(),
                ],
            }),
        ],
    }));
}

#[test]
fn nested_tags() {
    let tag = snax!(
        <div>
            <span>
            </span>
        </div>
    );

    compare(tag, HtmlContent::Tag(HtmlTag {
        name: Cow::Borrowed("div"),
        attributes: HashMap::new(),
        children: vec![
            HtmlContent::Tag(HtmlTag {
                name: Cow::Borrowed("span"),
                attributes: HashMap::new(),
                children: Vec::new(),
            }),
        ],
    }));
}

#[test]
fn adjacent_tags() {
    let tag = snax!(
        <div>
            <span></span>
            <div></div>
        </div>
    );

    compare(&tag, &HtmlContent::Tag(HtmlTag {
        name: Cow::Borrowed("div"),
        attributes: HashMap::new(),
        children: vec![
            HtmlContent::Tag(HtmlTag {
                name: Cow::Borrowed("span"),
                attributes: HashMap::new(),
                children: Vec::new(),
            }),
            HtmlContent::Tag(HtmlTag {
                name: Cow::Borrowed("div"),
                attributes: HashMap::new(),
                children: Vec::new(),
            }),
        ],
    }));
}

#[test]
fn nested_tags_self_closing() {
    let tag = snax!(
        <div>
            <span />
        </div>
    );

    compare(tag, HtmlContent::Tag(HtmlTag {
        name: Cow::Borrowed("div"),
        attributes: HashMap::new(),
        children: vec![
            HtmlContent::SelfClosingTag(HtmlSelfClosingTag {
                name: Cow::Borrowed("span"),
                attributes: HashMap::new(),
            }),
        ],
    }));
}

#[test]
fn borrow_content_nonstatic() {
    let foo = "hello".to_string();

    let tag = snax!(
        <span>
            { foo.as_str() }
        </span>
    );

    compare(tag, HtmlContent::Tag(HtmlTag {
        name: Cow::Borrowed("span"),
        attributes: HashMap::new(),
        children: vec![
            "hello".into(),
        ],
    }))
}

#[test]
fn borrow_attribute_nonstatic() {
    let foo = "world".to_string();

    let tag = snax!(
        <span hello={ foo.as_str() } />
    );

    compare(tag, HtmlContent::SelfClosingTag(HtmlSelfClosingTag {
        name: Cow::Borrowed("span"),
        attributes: hashmap! {
            Cow::Borrowed("hello") => Cow::Borrowed("world"),
        },
    }));
}