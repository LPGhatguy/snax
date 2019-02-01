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
pub struct HtmlTag<'a> {
    pub name: Cow<'a, str>,
    pub attributes: HashMap<Cow<'a, str>, Cow<'a, str>>,
    pub children: Vec<HtmlContent<'a>>,
}

impl<'a> HtmlTag<'a> {
    pub fn add_child<T: Into<HtmlContent<'a>>>(&mut self, child: T) {
        for item in child.into() {
            self.children.push(item);
        }
    }
}

impl<'a> fmt::Display for HtmlTag<'a> {
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
pub struct HtmlSelfClosingTag<'a> {
    pub name: Cow<'a, str>,
    pub attributes: HashMap<Cow<'a, str>, Cow<'a, str>>,
}

impl<'a> fmt::Display for HtmlSelfClosingTag<'a> {
    fn fmt(&self, output: &mut fmt::Formatter) -> fmt::Result {
        write!(output, "<{}", self.name)?;

        for (key, value) in &self.attributes {
            write!(output, " {}=\"{}\"", key, htmlescape::encode_minimal(value))?;
        }

        write!(output, "/>")
    }
}

#[derive(Debug, PartialEq)]
pub struct Fragment<'a> {
    children: Vec<HtmlContent<'a>>,
}

impl<'a> Fragment<'a> {
    pub fn new<T>(iter: T) -> Fragment<'a>
        where T: IntoIterator,
              T::Item: Into<HtmlContent<'a>>,
    {
        Fragment {
            children: iter.into_iter().map(Into::into).collect(),
        }
    }
}

impl<'a> FromIterator<HtmlContent<'a>> for Fragment<'a> {
    fn from_iter<I: IntoIterator<Item = HtmlContent<'a>>>(iter: I) -> Fragment<'a> {
        Fragment {
            children: iter.into_iter().collect(),
        }
    }
}

impl<'a> IntoIterator for Fragment<'a> {
    type Item = HtmlContent<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.into_iter()
    }
}

impl<'a> fmt::Display for Fragment<'a> {
    fn fmt(&self, output: &mut fmt::Formatter) -> fmt::Result {
        for child in &self.children {
            write!(output, "{}", child)?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct EscapedText<'a> {
    text: Cow<'a, str>,
}

impl<'a> fmt::Display for EscapedText<'a> {
    fn fmt(&self, output: &mut fmt::Formatter) -> fmt::Result {
        write!(output, "{}", htmlescape::encode_minimal(&self.text))
    }
}

#[derive(Debug, PartialEq)]
pub struct UnescapedText<'a> {
    text: Cow<'a, str>,
}

impl<'a> UnescapedText<'a> {
    pub fn new<T: Into<Cow<'a, str>>>(value: T) -> UnescapedText<'a> {
        UnescapedText {
            text: value.into(),
        }
    }
}

impl<'a> fmt::Display for UnescapedText<'a> {
    fn fmt(&self, output: &mut fmt::Formatter) -> fmt::Result {
        write!(output, "{}", self.text)
    }
}

#[derive(Debug, PartialEq)]
pub enum HtmlContent<'a> {
    Tag(HtmlTag<'a>),
    SelfClosingTag(HtmlSelfClosingTag<'a>),
    EscapedText(EscapedText<'a>),
    UnescapedText(UnescapedText<'a>),
    Fragment(Fragment<'a>),
    None,
}

pub enum HtmlContentIntoIter<'a> {
    Once(std::iter::Once<HtmlContent<'a>>),
    Children(std::vec::IntoIter<HtmlContent<'a>>),
    None,
}

impl<'a> Iterator for HtmlContentIntoIter<'a> {
    type Item = HtmlContent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            HtmlContentIntoIter::Once(inner) => inner.next(),
            HtmlContentIntoIter::Children(inner) => inner.next(),
            HtmlContentIntoIter::None => None,
        }
    }
}

impl<'a> IntoIterator for HtmlContent<'a> {
    type Item = HtmlContent<'a>;
    type IntoIter = HtmlContentIntoIter<'a>;

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

impl<'a> fmt::Display for HtmlContent<'a> {
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

impl<'a> From<HtmlTag<'a>> for HtmlContent<'a> {
    fn from(tag: HtmlTag<'a>) -> HtmlContent<'a> {
        HtmlContent::Tag(tag)
    }
}

impl<'a> From<HtmlSelfClosingTag<'a>> for HtmlContent<'a> {
    fn from(tag: HtmlSelfClosingTag<'a>) -> HtmlContent<'a> {
        HtmlContent::SelfClosingTag(tag)
    }
}

impl<'a> From<Fragment<'a>> for HtmlContent<'a> {
    fn from(tag: Fragment<'a>) -> HtmlContent<'a> {
        HtmlContent::Fragment(tag)
    }
}

impl<'a> From<UnescapedText<'a>> for HtmlContent<'a> {
    fn from(tag: UnescapedText<'a>) -> HtmlContent<'a> {
        HtmlContent::UnescapedText(tag)
    }
}

impl<'a> From<Option<HtmlContent<'a>>> for HtmlContent<'a> {
    fn from(value: Option<HtmlContent<'a>>) -> HtmlContent<'a> {
        match value {
            Some(HtmlContent::None) => HtmlContent::None,
            Some(content) => content,
            None => HtmlContent::None,
        }
    }
}

impl<'a> From<&'a str> for HtmlContent<'a> {
    fn from(value: &'a str) -> HtmlContent<'a> {
        HtmlContent::EscapedText(EscapedText {
            text: value.into(),
        })
    }
}

impl From<String> for HtmlContent<'static> {
    fn from(value: String) -> HtmlContent<'static> {
        HtmlContent::EscapedText(EscapedText {
            text: value.into(),
        })
    }
}

impl<'a> From<Cow<'a, str>> for HtmlContent<'a> {
    fn from(value: Cow<'a, str>) -> HtmlContent<'a> {
        HtmlContent::EscapedText(EscapedText {
            text: value,
        })
    }
}

impl<'a> From<&'a &'static str> for HtmlContent<'static> {
    fn from(value: &'a &'static str) -> HtmlContent<'static> {
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

        assert_eq!(tag, HtmlContent::Tag(HtmlTag {
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

        assert_eq!(tag, HtmlContent::SelfClosingTag(HtmlSelfClosingTag {
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
}
