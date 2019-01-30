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

impl<T> From<T> for HtmlContent where T: Into<Cow<'static, str>> {
    fn from(stringish: T) -> HtmlContent {
        HtmlContent::Text(stringish.into())
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
    fn literal_content() {
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
}
