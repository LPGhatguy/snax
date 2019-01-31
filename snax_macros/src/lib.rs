#![recursion_limit="128"]

extern crate proc_macro;

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

    let parsed_content = parse_root(input.into_iter())
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

#[derive(Debug, Clone)]
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
            panic!("Expected end of Snax macro, got {}", token);
        },
    }
}

fn parse_root(mut input: impl Iterator<Item = TokenTree>) -> Result<HtmlContent, TagError> {
    let mut tag_stack: Vec<(HtmlOpenToken, Vec<HtmlContent>)> = Vec::new();

    loop {
        match parse_html_token(&mut input)? {
            HtmlToken::OpenTag(opening_tag) => {
                tag_stack.push((opening_tag, Vec::new()));
            },
            HtmlToken::CloseTag(closing_tag) => {
                let (opening_tag, children) = tag_stack.pop()
                    .ok_or_else(|| TagError::UnexpectedHtmlToken(HtmlToken::CloseTag(closing_tag.clone())))?;

                assert_eq!(opening_tag.name, closing_tag.name);

                let tag = HtmlTag {
                    name: opening_tag.name,
                    attributes: opening_tag.attributes,
                    children,
                };

                match tag_stack.last_mut() {
                    None => {
                        assert_is_end(&mut input);
                        return Ok(HtmlContent::Tag(tag));
                    },
                    Some((_, parent_children)) => {
                        parent_children.push(HtmlContent::Tag(tag));
                    },
                }
            },
            HtmlToken::SelfClosingTag(self_closing_tag) => {
                let tag = HtmlSelfClosingTag {
                    name: self_closing_tag.name,
                    attributes: self_closing_tag.attributes,
                };

                match tag_stack.last_mut() {
                    None => {
                        assert_is_end(&mut input);
                        return Ok(HtmlContent::SelfClosingTag(tag));
                    },
                    Some((_, parent_children)) => {
                        parent_children.push(HtmlContent::SelfClosingTag(tag));
                    },
                }
            },
            HtmlToken::Textish(textish) => {
                match tag_stack.last_mut() {
                    None => {
                        assert_is_end(&mut input);
                        return Ok(HtmlContent::Textish(textish.content));
                    },
                    Some((_, parent_children)) => {
                        parent_children.push(HtmlContent::Textish(textish.content));
                    },
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
                    let mut attributes = Vec::new();

                    loop {
                        match input.next().ok_or(TagError::UnexpectedEnd)? {
                            TokenTree::Ident(attribute_name) => {
                                yum!(input, TokenTree::Punct(ref punct) if punct.as_char() == '=');

                                match input.next().ok_or(TagError::UnexpectedEnd)? {
                                    content @ TokenTree::Literal(_) => attributes.push((attribute_name, content)),
                                    content @ TokenTree::Group(_) => attributes.push((attribute_name, content)),
                                    unexpected => return Err(TagError::UnexpectedToken(unexpected)),
                                }
                            },
                            TokenTree::Punct(ref punct) if punct.as_char() == '>' => {
                                // Opening tag

                                return Ok(HtmlToken::OpenTag(HtmlOpenToken {
                                    name,
                                    attributes,
                                }));
                            },
                            TokenTree::Punct(ref punct) if punct.as_char() == '/' => {
                                // Self-closing tag

                                yum!(input, TokenTree::Punct(ref punct) if punct.as_char() == '>');

                                return Ok(HtmlToken::SelfClosingTag(HtmlSelfClosingToken {
                                    name,
                                    attributes,
                                }));
                            },
                            unexpected => return Err(TagError::UnexpectedToken(unexpected)),
                        }
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
            __snax_attributes.insert(stringify!(#key).into(), #value.into());
        ))
        .collect();

    let tag_name = &tag.name;

    quote!(
        {
            let mut __snax_attributes = ::std::collections::HashMap::new();
            #attribute_insertions

            snax::HtmlContent::SelfClosingTag(snax::HtmlSelfClosingTag {
                name: ::std::borrow::Cow::Borrowed(stringify!(#tag_name)),
                attributes: __snax_attributes,
            })
        }
    )
}

fn emit_tag(tag: &HtmlTag) -> TokenStream {
    let attribute_insertions: TokenStream = tag.attributes
        .iter()
        .map(|(key, value)| quote!(
            __snax_attributes.insert(stringify!(#key).into(), #value.into());
        ))
        .collect();

    let child_insertions: TokenStream = tag.children
        .iter()
        .map(|child| {
            let emitted = emit_content(child);

            quote!(
                __snax_tag.add_child(#emitted);
            )
        })
        .collect();

    let tag_name = &tag.name;

    quote!(
        {
            let mut __snax_attributes = ::std::collections::HashMap::new();
            #attribute_insertions

            let mut __snax_tag = snax::HtmlTag {
                name: ::std::borrow::Cow::Borrowed(stringify!(#tag_name)),
                attributes: __snax_attributes,
                children: ::std::vec::Vec::new(),
            };

            #child_insertions

            snax::HtmlContent::Tag(__snax_tag)
        }
    )
}

fn emit_textish(tt: &TokenTree) -> TokenStream {
    [tt].iter().cloned().cloned().collect()
}