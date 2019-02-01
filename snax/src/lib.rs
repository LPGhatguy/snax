use std::{
    collections::HashMap,
    borrow::Cow,
    iter::FromIterator,
    fmt,
};

use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack(support_nested)]
pub use snax_macros::snax;

#[derive(Debug, PartialEq)]
pub struct HtmlTag {
    pub name: Cow<'static, str>,
    pub attributes: HashMap<Cow<'static, str>, Cow<'static, str>>,
    pub children: Vec<HtmlContent>,
}

impl HtmlTag {
    pub fn add_child<T: Into<HtmlContent>>(&mut self, child: T) {
        for item in child.into() {
            self.children.push(item);
        }
    }
}

impl fmt::Display for HtmlTag {
    fn fmt(&self, output: &mut fmt::Formatter) -> fmt::Result {
        write!(output, "<{}", self.name)?;

        for (key, value) in &self.attributes {
            write!(output, " {}=\"{}\"", key, htmlescape::encode_minimal(value))?;
        }

        write!(output, ">")?;

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
        write!(output, "<{}", self.name)?;

        for (key, value) in &self.attributes {
            write!(output, " {}=\"{}\"", key, htmlescape::encode_minimal(value))?;
        }

        write!(output, "/>")
    }
}

#[derive(Debug, PartialEq)]
pub struct Fragment {
    children: Vec<HtmlContent>,
}

impl Fragment {
    pub fn new<T>(iter: T) -> Fragment
        where T: IntoIterator,
              T::Item: Into<HtmlContent>,
    {
        Fragment {
            children: iter.into_iter().map(Into::into).collect(),
        }
    }
}

impl FromIterator<HtmlContent> for Fragment {
    fn from_iter<I: IntoIterator<Item = HtmlContent>>(iter: I) -> Fragment {
        Fragment {
            children: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for Fragment {
    type Item = HtmlContent;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.into_iter()
    }
}

impl fmt::Display for Fragment {
    fn fmt(&self, output: &mut fmt::Formatter) -> fmt::Result {
        for child in &self.children {
            write!(output, "{}", child)?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct EscapedText {
    text: Cow<'static, str>,
}

impl fmt::Display for EscapedText {
    fn fmt(&self, output: &mut fmt::Formatter) -> fmt::Result {
        write!(output, "{}", htmlescape::encode_minimal(&self.text))
    }
}

#[derive(Debug, PartialEq)]
pub struct UnescapedText {
    text: Cow<'static, str>,
}

impl UnescapedText {
    pub fn new<T: Into<Cow<'static, str>>>(value: T) -> UnescapedText {
        UnescapedText {
            text: value.into(),
        }
    }
}

impl fmt::Display for UnescapedText {
    fn fmt(&self, output: &mut fmt::Formatter) -> fmt::Result {
        write!(output, "{}", self.text)
    }
}

#[derive(Debug, PartialEq)]
pub enum HtmlContent {
    Tag(HtmlTag),
    SelfClosingTag(HtmlSelfClosingTag),
    EscapedText(EscapedText),
    UnescapedText(UnescapedText),
    Fragment(Fragment),
    None,
}

pub enum HtmlContentIntoIter {
    Once(std::iter::Once<HtmlContent>),
    Children(std::vec::IntoIter<HtmlContent>),
    None,
}

impl Iterator for HtmlContentIntoIter {
    type Item = HtmlContent;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            HtmlContentIntoIter::Once(inner) => inner.next(),
            HtmlContentIntoIter::Children(inner) => inner.next(),
            HtmlContentIntoIter::None => None,
        }
    }
}

impl IntoIterator for HtmlContent {
    type Item = HtmlContent;
    type IntoIter = HtmlContentIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            HtmlContent::Tag(_) |
            HtmlContent::SelfClosingTag(_) |
            HtmlContent::EscapedText(_) |
            HtmlContent::UnescapedText(_) => {
                HtmlContentIntoIter::Once(std::iter::once(self))
            },
            HtmlContent::Fragment(Fragment { children }) => {
                HtmlContentIntoIter::Children(children.into_iter())
            },
            HtmlContent::None => HtmlContentIntoIter::None,
        }
    }
}

impl fmt::Display for HtmlContent {
    fn fmt(&self, output: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HtmlContent::Tag(tag) => write!(output, "{}", tag),
            HtmlContent::SelfClosingTag(tag) => write!(output, "{}", tag),
            HtmlContent::EscapedText(text) => write!(output, "{}", text),
            HtmlContent::UnescapedText(text) => write!(output, "{}", text),
            HtmlContent::Fragment(fragment) => write!(output, "{}", fragment),
            HtmlContent::None => Ok(()),
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

impl From<Fragment> for HtmlContent {
    fn from(tag: Fragment) -> HtmlContent {
        HtmlContent::Fragment(tag)
    }
}

impl From<UnescapedText> for HtmlContent {
    fn from(tag: UnescapedText) -> HtmlContent {
        HtmlContent::UnescapedText(tag)
    }
}

impl From<Option<HtmlContent>> for HtmlContent {
    fn from(value: Option<HtmlContent>) -> HtmlContent {
        match value {
            Some(HtmlContent::None) => HtmlContent::None,
            Some(content) => content,
            None => HtmlContent::None,
        }
    }
}

impl From<&'static str> for HtmlContent {
    fn from(value: &'static str) -> HtmlContent {
        HtmlContent::EscapedText(EscapedText {
            text: value.into(),
        })
    }
}

impl From<String> for HtmlContent {
    fn from(value: String) -> HtmlContent {
        HtmlContent::EscapedText(EscapedText {
            text: value.into(),
        })
    }
}

impl From<Cow<'static, str>> for HtmlContent {
    fn from(value: Cow<'static, str>) -> HtmlContent {
        HtmlContent::EscapedText(EscapedText {
            text: value,
        })
    }
}

impl<'a> From<&'a &'static str> for HtmlContent {
    fn from(value: &'a &'static str) -> HtmlContent {
        HtmlContent::EscapedText(EscapedText {
            text: Cow::Borrowed(*value),
        })
    }
}

#[cfg(test)]
mod test {
    use std::{
        collections::HashMap,
        borrow::{Borrow, Cow},
    };

    use maplit::hashmap;

    use crate::{
        HtmlTag,
        HtmlSelfClosingTag,
        HtmlContent,
        Fragment,
    };

    use crate as snax;

    fn compare<A, B>(a: A, b: B)
        where A: Borrow<HtmlContent>,
              B: Borrow<HtmlContent>,
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
                "Hello, world!".into(),
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

        assert_eq!(tag, HtmlContent::Tag(HtmlTag {
            name: Cow::Borrowed("span"),
            attributes: HashMap::new(),
            children: vec![
                "hello".into(),
                "world".into(),
            ],
        }));
    }

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

        assert_eq!(tag, HtmlContent::Tag(HtmlTag {
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
