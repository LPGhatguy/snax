//! Snax is [JSX][jsx-intro] for Rust.
//!
//! More specifically, it's a library for proc macro authors who want JSX-like
//! syntax in their libraries.
//!
//! For the current best example of how to use Snax, check out the [current
//! source of the ritz crate][ritz-github].
//!
//! ## Requirements
//! Snax requires Rust 1.32 or newer.
//!
//! ## License
//! Snax is available under the MIT license. See [LICENSE.txt](LICENSE.txt) for
//! details.
//!
//! [jsx-intro]: https://reactjs.org/docs/introducing-jsx.html
//! [ritz-github]: https://github.com/LPGhatguy/ritz

mod tokenizer;

use proc_macro2::{
    TokenStream,
    TokenTree,
    Ident,
};

use crate::tokenizer::{
    HtmlToken,
    HtmlOpenToken,
    TokenizeError,
    parse_html_token,
};

/// An attribute that's present on either a [`SnaxTag`] or a
/// [`SnaxSelfClosingTag`].
///
/// Attributes can only be `Simple` right now, which is a name-value pair where
/// the name is a fixed ident and the value is either a Literal or a Group.
///
/// In the future, snax_syntax will support attribute spreading. See [issue
/// #4](https://github.com/LPGhatguy/snax/issues/4) for more details and
/// progress updates.
///
/// [`SnaxTag`]: struct.SnaxTag.html
/// [`SnaxSelfClosingTag`]: struct.SnaxSelfClosingTag.html
#[derive(Debug)]
pub enum SnaxAttribute {
    /// A name-value pair describing a property.
    ///
    /// ```html
    /// <div foo="bar" />
    ///      ^^^^^^^^^
    ///      SnaxAttribute::Simple {
    ///          name: Ident(foo),
    ///          value: TokenTree("bar"),
    ///      }
    /// ```
    ///
    /// ```html
    /// <div hello={ "world" }>"hey there"</div>
    ///      ^^^^^^^^^^^^^^^^^
    ///      SnaxAttribute::Simple {
    ///          name: Ident(hello),
    ///          value: TokenTree({ "world" }),
    ///      }
    /// ```
    Simple {
        name: Ident,
        value: TokenTree,
    },
}

/// One complete block in the syntax.
///
/// For more information, look at the documentation for the struct that each
/// variant wraps.
#[derive(Debug)]
pub enum SnaxItem {
    /// A standard tag, which can have attributes and children.
    Tag(SnaxTag),

    /// An empty tag, which can only have attributes.
    SelfClosingTag(SnaxSelfClosingTag),

    /// A fragment, containing a list of zero or more children.
    Fragment(SnaxFragment),

    /// A block of content, which can be a literal or a group that evaluates to
    /// an `Into<SnaxItem>` at runtime.
    Content(TokenTree),
}

/// A standard tag, which can have attributes and children.
///
/// ```html
/// <div hello="world">"Hey!"</div>
/// ```
#[derive(Debug)]
pub struct SnaxTag {
    pub name: Ident,
    pub attributes: Vec<SnaxAttribute>,
    pub children: Vec<SnaxItem>,
}

/// A self-closing tag, which doesn't have children:
///
/// ```html
/// <meta name="foo" value="bar" />
/// ```
///
/// Note that snax_syntax does not support automatically closing unclosed
/// tags like HTML does, such as `<br>`. These tags need to be written as
/// `<br />` in order to simplify parsing.
#[derive(Debug)]
pub struct SnaxSelfClosingTag {
    pub name: Ident,
    pub attributes: Vec<SnaxAttribute>,
}

/// A fragment, which only contains children.
///
/// ```html
/// <>
///     <span>Hey</span>
///     <span>there!</span>
/// </>
/// ```
///
/// This syntax comes from JSX, and in frameworks like React, it's expected that
/// the children of a fragment will be merged into the fragment's parent.
#[derive(Debug)]
pub struct SnaxFragment {
    pub children: Vec<SnaxItem>,
}

#[derive(Debug)]
pub enum ParseError {
    Tokenize(TokenizeError),
    UnexpectedItem(HtmlToken),
    UnexpectedToken(TokenTree),
}

impl From<TokenizeError> for ParseError {
    fn from(error: TokenizeError) -> ParseError {
        ParseError::Tokenize(error)
    }
}

macro_rules! expect_end {
    ($iterator: expr) => {
        match $iterator.next() {
            None => {},
            Some(unexpected) => return Err(ParseError::UnexpectedToken(unexpected)),
        }
    };
}

#[derive(Debug)]
enum OpenToken {
    Tag(HtmlOpenToken),
    Fragment,
}

/// Attempts to parse a `proc_macro2::TokenStream` into a `SnaxItem`.
pub fn parse(input_stream: TokenStream) -> Result<SnaxItem, ParseError> {
    let mut input = input_stream.into_iter();
    let mut tag_stack: Vec<(OpenToken, Vec<SnaxItem>)> = Vec::new();

    loop {
        match parse_html_token(&mut input)? {
            HtmlToken::OpenTag(opening_tag) => {
                tag_stack.push((OpenToken::Tag(opening_tag), Vec::new()));
            },
            HtmlToken::CloseTag(closing_tag) => {
                let (open_token, children) = tag_stack.pop()
                    .ok_or_else(|| ParseError::UnexpectedItem(HtmlToken::CloseTag(closing_tag.clone())))?;

                let opening_tag = match open_token {
                    OpenToken::Tag(tag) => tag,
                    OpenToken::Fragment => return Err(ParseError::UnexpectedItem(HtmlToken::CloseTag(closing_tag.clone()))),
                };

                assert_eq!(opening_tag.name, closing_tag.name);

                let tag = SnaxTag {
                    name: opening_tag.name,
                    attributes: opening_tag.attributes,
                    children,
                };

                match tag_stack.last_mut() {
                    None => {
                        expect_end!(input);
                        return Ok(SnaxItem::Tag(tag));
                    },
                    Some((_, parent_children)) => {
                        parent_children.push(SnaxItem::Tag(tag));
                    },
                }
            },
            HtmlToken::OpenFragment => {
                tag_stack.push((OpenToken::Fragment, Vec::new()));
            },
            HtmlToken::CloseFragment => {
                let (open_token, children) = tag_stack.pop()
                    .ok_or_else(|| ParseError::UnexpectedItem(HtmlToken::CloseFragment))?;

                match open_token {
                    OpenToken::Fragment => {},
                    OpenToken::Tag(_) => return Err(ParseError::UnexpectedItem(HtmlToken::CloseFragment)),
                }

                let fragment = SnaxFragment {
                    children,
                };

                match tag_stack.last_mut() {
                    None => {
                        expect_end!(input);
                        return Ok(SnaxItem::Fragment(fragment));
                    },
                    Some((_, parent_children)) => {
                        parent_children.push(SnaxItem::Fragment(fragment));
                    },
                }
            },
            HtmlToken::SelfClosingTag(self_closing_tag) => {
                let tag = SnaxSelfClosingTag {
                    name: self_closing_tag.name,
                    attributes: self_closing_tag.attributes,
                };

                match tag_stack.last_mut() {
                    None => {
                        expect_end!(input);
                        return Ok(SnaxItem::SelfClosingTag(tag));
                    },
                    Some((_, parent_children)) => {
                        parent_children.push(SnaxItem::SelfClosingTag(tag));
                    },
                }
            },
            HtmlToken::Textish(textish) => {
                match tag_stack.last_mut() {
                    None => {
                        expect_end!(input);
                        return Ok(SnaxItem::Content(textish.content));
                    },
                    Some((_, parent_children)) => {
                        parent_children.push(SnaxItem::Content(textish.content));
                    },
                }
            },
        }
    }
}