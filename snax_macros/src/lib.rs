#![recursion_limit="128"]

extern crate proc_macro;

use proc_macro_hack::proc_macro_hack;
use proc_macro2::{
    Literal,
    TokenStream,
    TokenTree,
};
use quote::quote;

use snax_syntax::{
    SnaxAttribute,
    SnaxFragment,
    SnaxItem,
    SnaxSelfClosingTag,
    SnaxTag,
};

#[proc_macro_hack]
pub fn snax(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);

    let parsed_content = snax_syntax::parse(input)
        .expect("Could not even");

    let output = emit_item(&parsed_content);

    proc_macro::TokenStream::from(output)
}

fn emit_item(item: &SnaxItem) -> TokenStream {
    match item {
        SnaxItem::Tag(tag) => emit_tag(tag),
        SnaxItem::SelfClosingTag(tag) => emit_self_closing_tag(tag),
        SnaxItem::Content(tt) => emit_content(tt),
        SnaxItem::Fragment(fragment) => emit_fragment(fragment),
    }
}

fn emit_attributes(attributes: &[SnaxAttribute]) -> TokenStream {
    attributes
        .iter()
        .map(|attribute| {
            match attribute {
                SnaxAttribute::Simple { name, value } => {
                    let name_literal = Literal::string(&name.to_string());

                    quote!(
                        __snax_tag.set_attribute(#name_literal, #value);
                    )
                },
            }
        })
        .collect()
}

fn emit_children(children: &[SnaxItem]) -> TokenStream {
    children
        .iter()
        .map(|child| {
            let emitted = emit_item(child);

            quote!(
                __snax_tag.add_child(#emitted);
            )
        })
        .collect()
}

fn emit_self_closing_tag(tag: &SnaxSelfClosingTag) -> TokenStream {
    let attribute_insertions = emit_attributes(&tag.attributes);
    let tag_name_literal = Literal::string(&tag.name.to_string());

    quote!(
        {
            let mut __snax_tag = ::snax::HtmlSelfClosingTag {
                name: ::std::borrow::Cow::Borrowed(#tag_name_literal),
                attributes: ::std::collections::HashMap::new(),
            };

            #attribute_insertions

            ::snax::HtmlContent::SelfClosingTag(__snax_tag)
        }
    )
}

fn emit_tag(tag: &SnaxTag) -> TokenStream {
    let attribute_insertions = emit_attributes(&tag.attributes);
    let child_insertions = emit_children(&tag.children);
    let tag_name_literal = Literal::string(&tag.name.to_string());

    quote!(
        {
            let mut __snax_tag = ::snax::HtmlTag {
                name: ::std::borrow::Cow::Borrowed(#tag_name_literal),
                attributes: ::std::collections::HashMap::new(),
                children: ::std::vec::Vec::new(),
            };

            #attribute_insertions
            #child_insertions

            ::snax::HtmlContent::Tag(__snax_tag)
        }
    )
}

fn emit_fragment(fragment: &SnaxFragment) -> TokenStream {
    let child_insertions = emit_children(&fragment.children);

    quote!(
        {
            let mut __snax_tag = ::snax::Fragment {
                children: ::std::vec::Vec::new(),
            };

            #child_insertions

            ::snax::HtmlContent::Fragment(__snax_tag)
        }
    )
}

fn emit_content(tt: &TokenTree) -> TokenStream {
    quote!(
        ::snax::HtmlContent::from(#tt)
    )
}