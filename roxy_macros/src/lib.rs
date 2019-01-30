#![recursion_limit="128"]

extern crate proc_macro;

use std::mem;

use proc_macro_hack::proc_macro_hack;
use proc_macro2::{
    TokenStream,
    TokenTree,
    Ident,
};
use quote::quote;

#[proc_macro_hack]
pub fn snax(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);

    // println!("{:#?}", input);

    let parsed_content = parse_root(input.into_iter().peekable())
        .expect("Could not even");

    let output = emit_content(&parsed_content);

    proc_macro::TokenStream::from(output)
}

macro_rules! yum {
    ($iterator: expr, $pattern: pat $(if $guard: expr)? => $result: expr) => {
        match $iterator.next().ok_or(TagError::UnexpectedEnd)? {
            $pattern $(if $guard)? => $result,
            unexpected => return Err(TagError::UnexpectedToken(unexpected)),
        }
    };
    ($iterator: expr, $pattern: pat $(if $guard: expr)?) => {
        yum!($iterator, $pattern $(if $guard)? => ())
    };
}

#[derive(Debug)]
enum HtmlContent {
    Tag(HtmlTag),
    SelfClosingTag(HtmlSelfClosingTag),
    Textish(TokenTree),
}

#[derive(Debug)]
struct HtmlTag {
    name: Ident,
    attributes: Vec<(Ident, TokenTree)>,
    children: Vec<HtmlContent>,
}

#[derive(Debug)]
struct HtmlSelfClosingTag {
    name: Ident,
    attributes: Vec<(Ident, TokenTree)>,
}

#[derive(Debug)]
enum TagError {
    // TODO: Span?
    UnexpectedEnd,
    UnexpectedToken(TokenTree),
    UnexpectedHtmlToken(HtmlToken),
}

#[derive(Debug)]
enum HtmlToken {
    OpenTag(HtmlOpenToken),
    CloseTag(HtmlCloseToken),
    SelfClosingTag(HtmlSelfClosingToken),
    Textish(HtmlTextishToken),
}

#[derive(Debug)]
struct HtmlOpenToken {
    name: Ident,
    attributes: Vec<(Ident, TokenTree)>,
}

#[derive(Debug)]
struct HtmlCloseToken {
    name: Ident,
}

#[derive(Debug)]
struct HtmlSelfClosingToken {
    name: Ident,
    attributes: Vec<(Ident, TokenTree)>,
}

#[derive(Debug)]
struct HtmlTextishToken {
    content: TokenTree,
}

fn assert_is_end(mut input: impl Iterator<Item = TokenTree>) {
    match input.next() {
        None => {},
        Some(token) => {
            panic!("Expected end of Roxy macro, got {}", token);
        },
    }
}

fn parse_root(mut input: impl Iterator<Item = TokenTree>) -> Result<HtmlContent, TagError> {
    let mut tag_stack: Vec<HtmlOpenToken> = Vec::new();
    let mut children: Vec<HtmlContent> = Vec::new();

    loop {
        match parse_html_token(&mut input)? {
            HtmlToken::OpenTag(opening_tag) => {
                tag_stack.push(opening_tag);
            },
            HtmlToken::CloseTag(closing_tag) => {
                let opening_tag = tag_stack.pop()
                    .expect("Unexpected closing tag with no matching opening tag");

                assert_eq!(opening_tag.name, closing_tag.name);

                let tag = HtmlTag {
                    name: opening_tag.name,
                    attributes: opening_tag.attributes,
                    children: mem::replace(&mut children, Vec::new()),
                };

                if tag_stack.is_empty() {
                    assert_is_end(&mut input);
                    return Ok(HtmlContent::Tag(tag))
                } else {
                    children.push(HtmlContent::Tag(tag));
                }
            },
            HtmlToken::SelfClosingTag(self_closing_tag) => {
                let tag = HtmlSelfClosingTag {
                    name: self_closing_tag.name,
                    attributes: self_closing_tag.attributes,
                };

                if tag_stack.is_empty() {
                    assert_is_end(&mut input);
                    return Ok(HtmlContent::SelfClosingTag(tag))
                } else {
                    children.push(HtmlContent::SelfClosingTag(tag));
                }
            },
            HtmlToken::Textish(textish) => {
                if tag_stack.is_empty() {
                    assert_is_end(&mut input);
                    return Ok(HtmlContent::Textish(textish.content));
                } else {
                    children.push(HtmlContent::Textish(textish.content));
                }
            },
        }
    }
}

fn parse_html_token(mut input: impl Iterator<Item = TokenTree>) -> Result<HtmlToken, TagError> {
    match input.next().ok_or(TagError::UnexpectedEnd)? {
        TokenTree::Punct(ref punct) if punct.as_char() == '<' => {
            match input.next().ok_or(TagError::UnexpectedEnd)? {
                TokenTree::Punct(ref punct) if punct.as_char() == '/' => {
                    let name = yum!(input, TokenTree::Ident(ident) => ident);
                    yum!(input, TokenTree::Punct(ref punct) if punct.as_char() == '>');

                    Ok(HtmlToken::CloseTag(HtmlCloseToken {
                        name,
                    }))
                },
                TokenTree::Ident(name) => {
                    // TODO: Parse attributes
                    let attributes = Vec::new();

                    match input.next().ok_or(TagError::UnexpectedEnd)? {
                        TokenTree::Punct(ref punct) if punct.as_char() == '>' => {
                            // Opening tag

                            Ok(HtmlToken::OpenTag(HtmlOpenToken {
                                name,
                                attributes,
                            }))
                        },
                        TokenTree::Punct(ref punct) if punct.as_char() == '/' => {
                            // Self-closing tag

                            yum!(input, TokenTree::Punct(ref punct) if punct.as_char() == '>');

                            Ok(HtmlToken::SelfClosingTag(HtmlSelfClosingToken {
                                name,
                                attributes,
                            }))
                        },
                        unexpected => return Err(TagError::UnexpectedToken(unexpected)),
                    }
                },
                unexpected => return Err(TagError::UnexpectedToken(unexpected)),
            }
        },
        content @ TokenTree::Literal(_) => Ok(HtmlToken::Textish(HtmlTextishToken { content })),
        content @ TokenTree::Group(_) => Ok(HtmlToken::Textish(HtmlTextishToken { content })),
        unexpected => return Err(TagError::UnexpectedToken(unexpected)),
    }
}

fn emit_content(content: &HtmlContent) -> TokenStream {
    match content {
        HtmlContent::Tag(tag) => emit_tag(tag),
        HtmlContent::SelfClosingTag(tag) => emit_self_closing_tag(tag),
        HtmlContent::Textish(tt) => emit_textish(tt),
    }
}

fn emit_self_closing_tag(tag: &HtmlSelfClosingTag) -> TokenStream {
    let attribute_insertions: TokenStream = tag.attributes
        .iter()
        .map(|(key, value)| quote!(
            __roxy_attributes.insert(stringify!(#key), #value.into());
        ))
        .collect();

    let tag_name = &tag.name;

    quote!(
        {
            let mut __roxy_attributes = ::std::collections::HashMap::new();
            #attribute_insertions

            roxy::HtmlSelfClosingTag {
                name: std::borrow::Cow::Borrowed(stringify!(#tag_name)),
                attributes: __roxy_attributes,
            }
        }
    )
}

fn emit_tag(tag: &HtmlTag) -> TokenStream {
    let attribute_insertions: TokenStream = tag.attributes
        .iter()
        .map(|(key, value)| quote!(
            __roxy_attributes.insert(stringify!(#key), #value.into());
        ))
        .collect();

    let child_insertions: TokenStream = tag.children
        .iter()
        .map(|child| {
            let emitted = emit_content(child);

            quote!(
                __roxy_children.push(#emitted.into());
            )
        })
        .collect();

    let tag_name = &tag.name;

    quote!(
        {
            let mut __roxy_attributes = ::std::collections::HashMap::new();
            #attribute_insertions

            let mut __roxy_children = ::std::vec::Vec::new();
            #child_insertions

            roxy::HtmlTag {
                name: std::borrow::Cow::Borrowed(stringify!(#tag_name)),
                attributes: __roxy_attributes,
                children: __roxy_children,
            }
        }
    )
}

fn emit_textish(tt: &TokenTree) -> TokenStream {
    [tt].iter().cloned().cloned().collect()
}