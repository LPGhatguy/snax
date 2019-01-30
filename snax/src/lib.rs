use std::{
    collections::HashMap,
    borrow::Cow,
    fmt,
};

use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack]
pub use snax_macros::snax;

#[derive(Debug, PartialEq)]
pub struct HtmlTag {
    pub name: Cow<'static, str>,
    pub attributes: HashMap<Cow<'static, str>, Cow<'static, str>>,
    pub children: Vec<HtmlContent>,
}

impl fmt::Display for HtmlTag {
    fn fmt(&self, output: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Write attributes
        write!(output, "<{}>", self.name)?;

        for child in &self.children {
            write!(output, "{}", child)?;
        }

        write!(output, "</{}>", self.name)
    }
}

#[derive(Debug, PartialEq)]
pub struct HtmlSelfClosingTag {
    pub name: Cow<'static, str>,
    pub attributes: HashMap<Cow<'static, str>, Cow<'static, str>>,
}

impl fmt::Display for HtmlSelfClosingTag {
    fn fmt(&self, output: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Write attributes
        write!(output, "<{} />", self.name)
    }
}

#[derive(Debug, PartialEq)]
pub enum HtmlContent {
    Tag(HtmlTag),
    SelfClosingTag(HtmlSelfClosingTag),
    Text(Cow<'static, str>),
}

impl fmt::Display for HtmlContent {
    fn fmt(&self, output: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HtmlContent::Tag(tag) => write!(output, "{}", tag),
            HtmlContent::SelfClosingTag(tag) => write!(output, "{}", tag),
            HtmlContent::Text(text) => write!(output, "{}", htmlescape::encode_minimal(text)),
        }
    }
}

impl From<HtmlTag> for HtmlContent {
    fn from(tag: HtmlTag) -> HtmlContent {
        HtmlContent::Tag(tag)
    }
}

impl From<HtmlSelfClosingTag> for HtmlContent {
    fn from(tag: HtmlSelfClosingTag) -> HtmlContent {
        HtmlContent::SelfClosingTag(tag)
    }
}

impl<T> From<T> for HtmlContent where T: Into<Cow<'static, str>> {
    fn from(displayable: T) -> HtmlContent {
        HtmlContent::Text(displayable.into())
    }
}

#[cfg(test)]
mod test {
    use std::{
        collections::HashMap,
        borrow::Cow,
    };

    use maplit::hashmap;

    use crate::{
        HtmlTag,
        HtmlSelfClosingTag,
        HtmlContent,
    };

    use crate as snax;

    #[test]
    fn empty() {
        let tag = snax!(<div></div>);

        assert_eq!(tag, HtmlContent::Tag(HtmlTag {
            name: Cow::Borrowed("div"),
            attributes: HashMap::new(),
            children: Vec::new(),
        }));
    }

    #[test]
    fn self_closing() {
        let tag = snax!(<div />);

        assert_eq!(tag, HtmlContent::SelfClosingTag(HtmlSelfClosingTag {
            name: Cow::Borrowed("div"),
            attributes: HashMap::new(),
        }));
    }

    #[test]
    fn empty_with_attributes() {
        let tag = snax!(<div foo="bar" baz="qux"></div>);

        assert_eq!(tag, HtmlContent::Tag(HtmlTag {
            name: Cow::Borrowed("div"),
            attributes: hashmap!(
                Cow::Borrowed("foo") => Cow::Borrowed("bar"),
                Cow::Borrowed("baz") => Cow::Borrowed("qux"),
            ),
            children: Vec::new(),
        }));
    }

    #[test]
    fn empty_with_block_attribute() {
        let tag = snax!(<div foo={ (5 + 5).to_string() }></div>);

        assert_eq!(tag, HtmlContent::Tag(HtmlTag {
            name: Cow::Borrowed("div"),
            attributes: hashmap!(
                Cow::Borrowed("foo") => Cow::Borrowed("10"),
            ),
            children: Vec::new(),
        }));
    }

    #[test]
    fn self_closing_with_attribute() {
        let tag = snax!(<div foo="hello" />);

        assert_eq!(tag, HtmlContent::SelfClosingTag(HtmlSelfClosingTag {
            name: Cow::Borrowed("div"),
            attributes: hashmap!(
                Cow::Borrowed("foo") => Cow::Borrowed("hello"),
            ),
        }));
    }

    #[test]
    fn literal_string() {
        let tag = snax!(
            <span>
                "Hello, world!"
            </span>
        );

        assert_eq!(tag, HtmlContent::Tag(HtmlTag {
            name: Cow::Borrowed("span"),
            attributes: HashMap::new(),
            children: vec![
                HtmlContent::Text(Cow::Borrowed("Hello, world!")),
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

        assert_eq!(tag, HtmlContent::Tag(HtmlTag {
            name: Cow::Borrowed("span"),
            attributes: HashMap::new(),
            children: vec![
                HtmlContent::Text(Cow::Borrowed("10")),
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

        assert_eq!(tag, HtmlContent::Tag(HtmlTag {
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
    fn nested_tags_self_closing() {
        let tag = snax!(
            <div>
                <span />
            </div>
        );

        assert_eq!(tag, HtmlContent::Tag(HtmlTag {
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
}
