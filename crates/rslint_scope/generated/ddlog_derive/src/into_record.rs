use super::{add_trait_bounds, get_rename};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_quote, spanned::Spanned, Attribute, Data, DataEnum, DataStruct, DeriveInput, Error,
    Fields, FieldsNamed, FieldsUnnamed, Ident, ImplGenerics, Index, Result, TypeGenerics,
    WhereClause,
};

pub fn into_record_inner(input: DeriveInput) -> Result<TokenStream> {
    // The name of the struct
    let struct_ident = input.ident;

    // Use the given rename provided by `#[ddlog(rename = "...")]` or `#[ddlog(into_record = "...")]`
    // as the name of the record, defaulting to the struct's ident if none is given
    let struct_record_name = get_rename("IntoRecord", "into_record", input.attrs.iter())?
        .unwrap_or_else(|| struct_ident.to_string());

    // Add the required trait bounds
    let generics = add_trait_bounds(
        input.generics,
        vec![parse_quote!(differential_datalog::record::IntoRecord)],
    );
    let generics = generics.split_for_impl();

    match input.data {
        // Derive for structs
        Data::Struct(derive_struct) => {
            into_record_struct(struct_ident, struct_record_name, derive_struct, generics)
        }

        // Derive for enums
        Data::Enum(derive_enum) => {
            into_record_enum(struct_ident, struct_record_name, derive_enum, generics)
        }

        // Unions can't safely/soundly be automatically implemented over,
        // the user will have to manually enforce invariants on it
        Data::Union(union) => Err(Error::new_spanned(
            union.union_token,
            "`IntoRecord` is not able to be automatically implemented on unions",
        )),
    }
}

fn into_record_struct(
    struct_ident: Ident,
    struct_record_name: String,
    derive_struct: DataStruct,
    (impl_generics, type_generics, where_clause): (
        ImplGenerics,
        TypeGenerics,
        Option<&WhereClause>,
    ),
) -> Result<TokenStream> {
    // Generate the code converting the fields of the struct into a record
    let (guard, generated_record) = match derive_struct.fields {
        Fields::Named(struct_elements) => {
            let (record, element_idents) =
                named_struct_record(&struct_record_name, &struct_elements)?;
            let guard = quote! {
                let #struct_ident { #( #element_idents ),* } = self;
            };

            (guard, record)
        }

        Fields::Unnamed(tuple_elements) => {
            let (record, element_indices) =
                tuple_struct_record(&struct_record_name, &tuple_elements);
            let guard = quote! {
                let #struct_ident(#( #element_indices ),*) = self;
            };

            (guard, record)
        }

        Fields::Unit => {
            let record = unit_struct_record(&struct_record_name);

            (TokenStream::new(), record)
        }
    };

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics differential_datalog::record::IntoRecord for #struct_ident #type_generics #where_clause {
            fn into_record(self) -> differential_datalog::record::Record {
                #guard
                #generated_record
            }
        }
    })
}

fn into_record_enum(
    enum_ident: Ident,
    enum_record_name: String,
    derive_enum: DataEnum,
    (impl_generics, type_generics, where_clause): (
        ImplGenerics,
        TypeGenerics,
        Option<&WhereClause>,
    ),
) -> Result<TokenStream> {
    // Generate the code for turning variants into records
    let generated_variants = derive_enum
        .variants
        .into_iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            let variant_record_name =
                rename_variant(variant_ident, &enum_record_name, variant.attrs.iter())?;

            let (match_guard, generated_record) = match variant.fields {
                Fields::Named(struct_elements) => {
                    let (record, element_idents) =
                        named_struct_record(&variant_record_name, &struct_elements)?;
                    let guard = quote! {
                        #enum_ident::#variant_ident { #( #element_idents ),* }
                    };

                    (guard, record)
                }

                Fields::Unnamed(tuple_elements) => {
                    let (record, element_indices) =
                        tuple_struct_record(&variant_record_name, &tuple_elements);
                    let guard = quote! {
                        #enum_ident::#variant_ident(#( #element_indices ),*)
                    };

                    (guard, record)
                }

                Fields::Unit => {
                    let record = unit_struct_record(&variant_record_name);
                    let guard = quote! { #enum_ident::#variant_ident };

                    (guard, record)
                }
            };

            Ok(quote! {
                #match_guard => #generated_record,
            })
        })
        .collect::<Result<TokenStream>>()?;

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics differential_datalog::record::IntoRecord for #enum_ident #type_generics #where_clause {
            fn into_record(self) -> differential_datalog::record::Record {
                match self {
                    #generated_variants
                }
            }
        }
    })
}

/// Use the given rename provided by `#[ddlog(rename = "...")]` or `#[ddlog(into_record = "...")]`
/// as the name of the variant, defaulting to the variant's ident if none is given
fn rename_variant<'a, I>(ident: &Ident, record_name: &str, attrs: I) -> Result<String>
where
    I: Iterator<Item = &'a Attribute> + 'a,
{
    Ok(get_rename("IntoRecord", "into_record", attrs)?
        .unwrap_or_else(|| format!("{}::{}", record_name, ident)))
}

fn named_struct_record<'a>(
    record_name: &str,
    struct_elements: &'a FieldsNamed,
) -> Result<(TokenStream, impl Iterator<Item = &'a Ident> + 'a)> {
    let elements = struct_elements.named.iter().map(|element| {
        let element_ident = element.ident.as_ref().expect("all of FieldsNamed's fields have idents");
        let element_type = &element.ty;
        let element_record_name = get_rename("IntoRecord", "into_record", element.attrs.iter())?
            .unwrap_or_else(|| element_ident.to_string());

        Ok(quote! {
            (
                std::borrow::Cow::Borrowed(#element_record_name),
                <#element_type as differential_datalog::record::IntoRecord>::into_record(#element_ident),
            ),
        })
    })
    .collect::<Result<TokenStream>>()?;

    let record = quote! {
        differential_datalog::record::Record::NamedStruct(
            std::borrow::Cow::Borrowed(#record_name),
            vec![#elements],
        )
    };

    let idents = struct_elements.named.iter().map(|element| {
        element
            .ident
            .as_ref()
            .expect("all of FieldsNamed's fields have idents")
    });

    Ok((record, idents))
}

fn tuple_struct_record<'a>(
    record_name: &str,
    tuple_elements: &'a FieldsUnnamed,
) -> (TokenStream, impl Iterator<Item = Ident> + 'a) {
    let elements = tuple_elements
        .unnamed
        .iter()
        .enumerate()
        .map(|(idx, element)| {
            (
                format_ident!(
                    "_{}",
                    Index {
                        index: idx as u32,
                        span: element.span(),
                    }
                ),
                element,
            )
        });

    let element_records = elements.clone().map(|(index, element)| {
        let element_type = &element.ty;

        quote! {
            <#element_type as differential_datalog::record::IntoRecord>::into_record(#index),
        }
    });

    let record = quote! {
        differential_datalog::record::Record::PosStruct(
            std::borrow::Cow::Borrowed(#record_name),
            vec![#( #element_records )*],
        )
    };

    (record, elements.map(|(index, _)| index))
}

fn unit_struct_record(record_name: &str) -> TokenStream {
    quote! {
        differential_datalog::record::Record::NamedStruct(
            std::borrow::Cow::Borrowed(#record_name),
            std::vec::Vec::new(),
        )
    }
}
