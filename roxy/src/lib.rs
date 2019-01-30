use std::{
    collections::HashMap,
    borrow::Cow,
};

use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack]
pub use roxy_macros::snax;

#[derive(Debug, PartialEq)]
pub struct HtmlTag {
    pub name: Cow<'static, str>,
    pub attributes: HashMap<Cow<'static, str>, Cow<'static, str>>,
    pub children: Vec<HtmlContent>,
}

#[derive(Debug, PartialEq)]
pub enum HtmlContent {
    Tag(HtmlTag),
    Text(Cow<'static, str>),
}

impl From<HtmlTag> for HtmlContent {
    fn from(tag: HtmlTag) -> HtmlContent {
        HtmlContent::Tag(tag)
    }
}

impl<T> From<T> for HtmlContent where T: std::fmt::Display {
    fn from(displayable: T) -> HtmlContent {
        HtmlContent::Text(Cow::Owned(format!("{}", displayable)))
    }
}

#[cfg(test)]
mod test {
    use std::{
        collections::HashMap,
        borrow::Cow,
    };

    use crate::{
        HtmlTag,
        HtmlContent,
    };

    use crate as roxy;

    #[test]
    fn empty() {
        let tag = snax!(<div></div>);

        assert_eq!(tag, HtmlTag {
            name: Cow::Borrowed("div"),
            attributes: HashMap::new(),
            children: Vec::new(),
        });
    }

    #[test]
    fn empty_self_closing() {
        let tag = snax!(<div />);

        assert_eq!(tag, HtmlTag {
            name: Cow::Borrowed("div"),
            attributes: HashMap::new(),
            children: Vec::new(),
        });
    }

    #[test]
    fn literal_string() {
        let tag = snax!(
            <span>
                "Hello, world!"
            </span>
        );

        assert_eq!(tag, HtmlTag {
            name: Cow::Borrowed("span"),
            attributes: HashMap::new(),
            children: vec![
                HtmlContent::Text(Cow::Borrowed("Hello, world!")),
            ],
        });
    }

    #[test]
    fn literal_block() {
        let tag = snax!(
            <span>
                { 5 + 5 }
            </span>
        );

        assert_eq!(tag, HtmlTag {
            name: Cow::Borrowed("span"),
            attributes: HashMap::new(),
            children: vec![
                HtmlContent::Text(Cow::Borrowed("10")),
            ],
        });
    }

    #[test]
    fn nested_tags() {
        let tag = snax!(
            <div>
                <span>
                </span>
            </div>
        );

        assert_eq!(tag, HtmlTag {
            name: Cow::Borrowed("div"),
            attributes: HashMap::new(),
            children: vec![
                HtmlContent::Tag(HtmlTag {
                    name: Cow::Borrowed("span"),
                    attributes: HashMap::new(),
                    children: Vec::new(),
                }),
            ],
        });
    }

    #[test]
    fn nested_tags_self_closing() {
        let tag = snax!(
            <div>
                <span />
            </div>
        );

        assert_eq!(tag, HtmlTag {
            name: Cow::Borrowed("div"),
            attributes: HashMap::new(),
            children: vec![
                HtmlContent::Tag(HtmlTag {
                    name: Cow::Borrowed("span"),
                    attributes: HashMap::new(),
                    children: Vec::new(),
                }),
            ],
        });
    }
}
