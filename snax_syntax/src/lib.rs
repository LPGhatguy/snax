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

pub use crate::tokenizer::SnaxAttribute;

#[derive(Debug)]
pub enum SnaxItem {
    Tag(SnaxTag),
    SelfClosingTag(SnaxSelfClosingTag),
    Content(TokenTree),
}

#[derive(Debug)]
pub struct SnaxTag {
    pub name: Ident,
    pub attributes: Vec<SnaxAttribute>,
    pub children: Vec<SnaxItem>,
}

#[derive(Debug)]
pub struct SnaxSelfClosingTag {
    pub name: Ident,
    pub attributes: Vec<SnaxAttribute>,
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

pub fn parse(input_stream: TokenStream) -> Result<SnaxItem, ParseError> {
    let mut input = input_stream.into_iter();
    let mut tag_stack: Vec<(HtmlOpenToken, Vec<SnaxItem>)> = Vec::new();

    loop {
        match parse_html_token(&mut input)? {
            HtmlToken::OpenTag(opening_tag) => {
                tag_stack.push((opening_tag, Vec::new()));
            },
            HtmlToken::CloseTag(closing_tag) => {
                let (opening_tag, children) = tag_stack.pop()
                    .ok_or_else(|| ParseError::UnexpectedItem(HtmlToken::CloseTag(closing_tag.clone())))?;

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