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
    Textish(TokenTree),
}

#[derive(Debug)]
struct HtmlTag {
    name: Ident,
    attributes: Vec<(Ident, TokenTree)>,
    children: Vec<HtmlContent>,
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
    OpenTag(HtmlOpenTag),
    CloseTag(HtmlCloseTag),
    SelfClosingTag(HtmlSelfClosingTag),
    Textish(HtmlTextish),
}

#[derive(Debug)]
struct HtmlOpenTag {
    name: Ident,
    attributes: Vec<(Ident, TokenTree)>,
}

#[derive(Debug)]
struct HtmlCloseTag {
    name: Ident,
}

#[derive(Debug)]
struct HtmlSelfClosingTag {
    name: Ident,
    attributes: Vec<(Ident, TokenTree)>,
}

#[derive(Debug)]
struct HtmlTextish {
    content: TokenTree,
}

fn parse_root(mut input: impl Iterator<Item = TokenTree>) -> Result<HtmlContent, TagError> {
    let mut tag_stack: Vec<HtmlOpenTag> = Vec::new();
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
                    // TODO: Assert that this is the end?
                    return Ok(HtmlContent::Tag(tag))
                } else {
                    children.push(HtmlContent::Tag(tag));
                }
            },
            HtmlToken::SelfClosingTag(self_closing_tag) => {
                let tag = HtmlTag {
                    name: self_closing_tag.name,
                    attributes: self_closing_tag.attributes,
                    children: Vec::new(),
                };

                if tag_stack.is_empty() {
                    // TODO: Assert that this is the end?
                    return Ok(HtmlContent::Tag(tag))
                } else {
                    children.push(HtmlContent::Tag(tag));
                }
            },
            HtmlToken::Textish(textish) => {
                if tag_stack.is_empty() {
                    // TODO: Assert that this is the end?
                    return Ok(HtmlContent::Textish(textish.content));
                } else {
                    children.push(HtmlContent::Textish(textish.content));
                }
            },
        }
    }

    // let (name, attributes) = match parse_html_token(input)? {
    //     HtmlToken::OpenTag { name, attributes } => (name, attributes),
    //     unexpected => return Err(TagError::UnexpectedHtmlToken(unexpected)),
    // };

    // Ok(HtmlTag {
    //     name,
    //     attributes: Vec::new(),
    //     children: Vec::new(),
    // })
}

fn parse_html_token(mut input: impl Iterator<Item = TokenTree>) -> Result<HtmlToken, TagError> {
    match input.next().ok_or(TagError::UnexpectedEnd)? {
        TokenTree::Punct(ref punct) if punct.as_char() == '<' => {
            match input.next().ok_or(TagError::UnexpectedEnd)? {
                TokenTree::Punct(ref punct) if punct.as_char() == '/' => {
                    let name = yum!(input, TokenTree::Ident(ident) => ident);
                    yum!(input, TokenTree::Punct(ref punct) if punct.as_char() == '>');

                    Ok(HtmlToken::CloseTag(HtmlCloseTag {
                        name,
                    }))
                },
                TokenTree::Ident(name) => {
                    // TODO: Parse attributes
                    let attributes = Vec::new();

                    yum!(input, TokenTree::Punct(ref punct) if punct.as_char() == '>');

                    Ok(HtmlToken::OpenTag(HtmlOpenTag {
                        name,
                        attributes,
                    }))
                },
                unexpected => return Err(TagError::UnexpectedToken(unexpected)),
            }
        },
        content @ TokenTree::Literal(_) => Ok(HtmlToken::Textish(HtmlTextish { content })),
        content @ TokenTree::Group(_) => Ok(HtmlToken::Textish(HtmlTextish { content })),
        unexpected => return Err(TagError::UnexpectedToken(unexpected)),
    }
}

fn emit_content(content: &HtmlContent) -> TokenStream {
    match content {
        HtmlContent::Tag(tag) => emit_tag(tag),
        HtmlContent::Textish(tt) => emit_textish(tt),
    }
}

fn emit_tag(tag: &HtmlTag) -> TokenStream {
    let attribute_insertions: TokenStream = tag.attributes
        .iter()
        .map(|(key, value)| quote!(
            __rsx_attributes.insert(stringify!(#key), #value.into());
        ))
        .collect();

    let child_insertions: TokenStream = tag.children
        .iter()
        .map(|child| {
            let emitted = emit_content(child);

            quote!(
                __rsx_children.push(#emitted.into());
            )
        })
        .collect();

    let tag_name = &tag.name;

    quote!(
        {
            let mut __rsx_attributes = ::std::collections::HashMap::new();
            #attribute_insertions

            let mut __rsx_children = ::std::vec::Vec::new();
            #child_insertions

            rsx::HtmlTag {
                name: std::borrow::Cow::Borrowed(stringify!(#tag_name)),
                attributes: __rsx_attributes,
                children: __rsx_children,
            }
        }
    )
}

fn emit_textish(tt: &TokenTree) -> TokenStream {
    [tt].iter().cloned().cloned().collect()
}