use proc_macro2::{
    TokenTree,
    Ident,
    Span,
};

use crate::SnaxAttribute;

#[derive(Debug)]
pub enum HtmlToken {
    OpenTag(HtmlOpenToken),
    CloseTag(HtmlCloseToken),
    SelfClosingTag(HtmlSelfClosingToken),
    Textish(HtmlTextishToken),
    OpenFragment,
    CloseFragment,
}

impl HtmlToken {
    pub fn what(&self) -> String {
        match self {
            HtmlToken::OpenTag(token) => format!("opening tag '{}'", token.name),
            HtmlToken::CloseTag(token) => format!("closing tag '{}'", token.name),
            HtmlToken::SelfClosingTag(token) => format!("self-closing tag '{}'", token.name),
            HtmlToken::Textish(token) => format!("content"),
            HtmlToken::OpenFragment => format!("opening fragment"),
            HtmlToken::CloseFragment => format!("closing fragment"),
        }
    }

    pub fn spans(&self) -> (Span, Span) {
        match self {
            HtmlToken::OpenTag(token) => (token.start.clone(), token.end.clone()),
            HtmlToken::CloseTag(token) => (token.start.clone(), token.end.clone()),
            HtmlToken::SelfClosingTag(token) => (token.name.span(), token.name.span()),
            HtmlToken::Textish(token) => (token.content.span(), token.content.span()),
            HtmlToken::OpenFragment => (Span::call_site(), Span::call_site()),
            HtmlToken::CloseFragment => (Span::call_site(), Span::call_site()),
        }
    }
}

#[derive(Debug)]
pub struct HtmlOpenToken {
    pub name: Ident,
    pub attributes: Vec<SnaxAttribute>,
    pub start: Span,
    pub end: Span,
}

#[derive(Debug, Clone)]
pub struct HtmlCloseToken {
    pub name: Ident,
    pub start: Span,
    pub end: Span,
}

#[derive(Debug)]
pub struct HtmlSelfClosingToken {
    pub name: Ident,
    pub attributes: Vec<SnaxAttribute>,
}

#[derive(Debug)]
pub struct HtmlTextishToken {
    pub content: TokenTree,
}

#[derive(Debug)]
pub enum TokenizeError {
    UnexpectedEnd,
    UnexpectedToken(TokenTree),
}

/// Grabs the next item of the iterator, handling the None case, and then makes
/// sure the given pattern matches.
macro_rules! expect_next {
    ($iterator: expr, $pattern: pat $(if $guard: expr)? => $result: expr) => {
        match $iterator.next().ok_or(TokenizeError::UnexpectedEnd)? {
            $pattern $(if $guard)? => $result,
            unexpected => return Err(TokenizeError::UnexpectedToken(unexpected)),
        }
    };
    ($iterator: expr, $pattern: pat $(if $guard: expr)?) => {
        expect_next!($iterator, $pattern $(if $guard)? => ())
    };
}

pub fn parse_html_token(mut input: impl Iterator<Item = TokenTree>) -> Result<HtmlToken, TokenizeError> {
    match input.next().ok_or(TokenizeError::UnexpectedEnd)? {
        TokenTree::Punct(ref start_punct) if start_punct.as_char() == '<' => {
            match input.next().ok_or(TokenizeError::UnexpectedEnd)? {
                TokenTree::Punct(ref slash_punct) if slash_punct.as_char() == '/' => {
                    match input.next().ok_or(TokenizeError::UnexpectedEnd)? {
                        TokenTree::Punct(ref punct) if punct.as_char() == '>' => {
                            Ok(HtmlToken::CloseFragment)
                        },
                        TokenTree::Ident(name) => {
                            let end_punct = expect_next!(input, TokenTree::Punct(ref punct) if punct.as_char() == '>' => punct.clone());

                            Ok(HtmlToken::CloseTag(HtmlCloseToken {
                                name,
                                start: start_punct.span(),
                                end: end_punct.span(),
                            }))
                        },
                        unexpected => return Err(TokenizeError::UnexpectedToken(unexpected)),
                    }
                },
                TokenTree::Punct(ref punct) if punct.as_char() == '>' => {
                    Ok(HtmlToken::OpenFragment)
                },
                TokenTree::Ident(name) => {
                    let mut attributes = Vec::new();

                    loop {
                        match input.next().ok_or(TokenizeError::UnexpectedEnd)? {
                            TokenTree::Ident(attribute_name) => {
                                expect_next!(input, TokenTree::Punct(ref punct) if punct.as_char() == '=');

                                match input.next().ok_or(TokenizeError::UnexpectedEnd)? {
                                    value @ TokenTree::Literal(_) | value @ TokenTree::Group(_) => {
                                        attributes.push(SnaxAttribute::Simple {
                                            name: attribute_name,
                                            value,
                                        });
                                    },
                                    unexpected => return Err(TokenizeError::UnexpectedToken(unexpected)),
                                }
                            },
                            TokenTree::Punct(ref end_punct) if end_punct.as_char() == '>' => {
                                // Opening tag

                                return Ok(HtmlToken::OpenTag(HtmlOpenToken {
                                    name,
                                    attributes,
                                    start: start_punct.span(),
                                    end: end_punct.span(),
                                }));
                            },
                            TokenTree::Punct(ref punct) if punct.as_char() == '/' => {
                                // Self-closing tag

                                expect_next!(input, TokenTree::Punct(ref punct) if punct.as_char() == '>');

                                return Ok(HtmlToken::SelfClosingTag(HtmlSelfClosingToken {
                                    name,
                                    attributes,
                                }));
                            },
                            unexpected => return Err(TokenizeError::UnexpectedToken(unexpected)),
                        }
                    }
                },
                unexpected => return Err(TokenizeError::UnexpectedToken(unexpected)),
            }
        },
        content @ TokenTree::Literal(_) => Ok(HtmlToken::Textish(HtmlTextishToken { content })),
        content @ TokenTree::Group(_) => Ok(HtmlToken::Textish(HtmlTextishToken { content })),
        unexpected => return Err(TokenizeError::UnexpectedToken(unexpected)),
    }
}