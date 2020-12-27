use std::fmt::{self, Display, Formatter};

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    Attribute, DeriveInput, Error, GenericParam, Generics, Ident, Index, Lit, LitInt,
    MetaNameValue, Result, TypeParamBound,
};

mod from_record;
mod into_record;

/// Allows deriving `FromRecord` for structs and enums
///
/// The struct/enum and individual fields or enum variants can be renamed with the
/// `#[ddlog(rename = "new name")]` or `#[ddlog(from_record = "new name")]` attributes
///
/// ```rust
/// # use ddlog_derive::FromRecord;
///
/// #[derive(FromRecord)]
/// #[ddlog(rename = "foo")]
/// struct Foo {
///     #[ddlog(rename = "baz")]
///     bar: u32,
/// }
/// ```
///
#[proc_macro_derive(FromRecord, attributes(ddlog))]
pub fn derive_from_record(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    from_record::from_record_inner(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Allows deriving `IntoRecord` for structs and enums
///
/// The struct/enum and individual fields or enum variants can be renamed with the
/// `#[ddlog(rename = "new name")]` or `#[ddlog(into_record = "new name")]` attributes
///
/// ```rust
/// # use ddlog_derive::IntoRecord;
///
/// #[derive(IntoRecord)]
/// #[ddlog(rename = "foo")]
/// struct Foo {
///     #[ddlog(rename = "baz")]
///     bar: u32,
/// }
/// ```
///
#[proc_macro_derive(IntoRecord, attributes(ddlog))]
pub fn derive_into_record(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    into_record::into_record_inner(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Add a trait bound to every generic, skipping the addition if the generic
/// already has the required trait bound
fn add_trait_bounds(mut generics: Generics, bounds: Vec<TypeParamBound>) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            for bound in bounds.iter() {
                if !type_param
                    .bounds
                    .iter()
                    .any(|type_bound| type_bound == bound)
                {
                    type_param.bounds.push(bound.clone());
                }
            }
        }
    }

    generics
}

/// Get the a rename from the current attributes, returning `Some` with the contents as a
/// string literal if there is one and `None` otherwise
///
/// `macro_name` should be the name of the derive macro this is being called for, it's
/// used for error reporting and `specific_attr` should be the derive-specific attribute's
/// name, most likely `from_record` or `into_record`.
///
/// If more than one matching attributes are found, an error will be returned for the user, ex.
///
/// ```compile_fail
/// #[derive(FromRecord)]
/// #[ddlog(rename = "Bar")]
/// #[ddlog(from_record = "Baz")]
/// struct Foo {}
/// ```
///
/// This errors because `rename` and `from_record` conflict for `from_record` implementations.
///
/// Additionally, unrecognized idents within the `ddlog` attribute will receive errors, such as
/// `#[ddlog(non_existant = "...")]` and values given to attributes that are not string literals
/// will also error out, like `#[ddlog(rename = 123)]`
///
fn get_rename<'a, I>(macro_name: &str, specific_attr: &str, attrs: I) -> Result<Option<String>>
where
    I: Iterator<Item = &'a Attribute> + 'a,
{
    let mut renames = attrs
        .filter(|attr| attr.path.is_ident("ddlog"))
        .map(|attr| attr.parse_args::<MetaNameValue>())
        .filter_map(|attr| match attr {
            Ok(attr) => {
                if attr.path.is_ident("rename") || attr.path.is_ident(specific_attr) {
                    Some(Ok((attr.span(), attr.lit)))

                // Ignore correct idents that aren't for the current derive
                } else if attr.path.is_ident("from_record") || attr.path.is_ident("into_record") {
                    None

                // Unrecognized idents within the ddlog attribute will be an error
                } else {
                    Some(Err(Error::new_spanned(
                        attr.path,
                        format!(
                            "unrecognized attribute, expected `rename` or `{}`",
                            specific_attr,
                        ),
                    )))
                }
            }
            Err(err) => Some(Err(err)),
        })
        .map(|lit| {
            lit.and_then(|(span, lit)| {
                if let Lit::Str(string) = lit {
                    Ok((span, string.value()))
                } else {
                    Err(Error::new_spanned(
                        lit,
                        format!(
                            "`{}` can only be renamed to string literal values",
                            macro_name,
                        ),
                    ))
                }
            })
        })
        .collect::<Result<Vec<(Span, String)>>>()?;

    if renames.is_empty() {
        Ok(None)
    } else if renames.len() == 1 {
        Ok(Some(renames.remove(0).1))
    } else {
        Err(Error::new(
            renames[0].0,
            format!(
                "got {} separate renames when only one is allowed",
                renames.len()
            ),
        ))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum IdentOrIndex {
    Ident(Ident),
    Index(Index),
}

impl Parse for IdentOrIndex {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            Ok(Self::Ident(input.parse()?))
        } else if lookahead.peek(LitInt) {
            Ok(Self::Index(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for IdentOrIndex {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Ident(ident) => ident.to_tokens(tokens),
            Self::Index(index) => index.to_tokens(tokens),
        }
    }
}

impl Display for IdentOrIndex {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Ident(ident) => Display::fmt(ident, f),
            Self::Index(index) => Display::fmt(&index.index, f),
        }
    }
}
