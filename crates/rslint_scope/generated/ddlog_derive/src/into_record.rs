use super::{add_trait_bounds, get_rename, IdentOrIndex};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_quote, spanned::Spanned, Data, DataEnum, DataStruct, DeriveInput, Error, Ident,
    ImplGenerics, Index, Result, TypeGenerics, WhereClause,
};

// TODO: Add the `positional = {true, false}` attribute to make a positional record
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
    // Generate the records for each field of the struct
    let struct_fields = derive_struct
        .fields
        .into_iter()
        .enumerate()
        .map(|(idx, field)| {
            let field_span = field.span();
            // Tuple structs have no field names, but instead use the tuple indexes
            let field_ident = field.ident.map_or_else(
                || IdentOrIndex::Index(Index { index: idx as u32, span: field_span }),
                IdentOrIndex::Ident,
            );
            let field_type = field.ty;

            // Use the given rename provided by `#[ddlog(rename = "...")]` or `#[ddlog(into_record = "...")]`
            // as the name of the field, defaulting to the field's ident if none is given
            let field_record_name = get_rename("IntoRecord", "into_record", field.attrs.iter())?
                .unwrap_or_else(|| field_ident.to_string());

            // Generate the field name and convert the field's value into a record
            Ok(quote! {
                (
                    std::borrow::Cow::Borrowed(#field_record_name),
                    <#field_type as differential_datalog::record::IntoRecord>::into_record(self.#field_ident),
                ),
            })
        })
        .collect::<Result<TokenStream>>()?;

    // Generate the actual code
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics differential_datalog::record::IntoRecord for #struct_ident #type_generics #where_clause {
            fn into_record(self) -> differential_datalog::record::Record {
                differential_datalog::record::Record::NamedStruct(
                    std::borrow::Cow::Borrowed(#struct_record_name),
                    std::vec![#struct_fields],
                )
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
    let variants = derive_enum
        .variants
        .into_iter()
        .map(|variant| {
            let variant_ident = variant.ident;

            // Use the given rename provided by `#[ddlog(rename = "...")]` or `#[ddlog(into_record = "...")]`
            // as the name of the variant, defaulting to the variant's ident if none is given
            let variant_record_name = get_rename("IntoRecord", "into_record", variant.attrs.iter())?
                .map_or_else(
                    || format!("{}::{}", enum_record_name, variant_ident),
                    |rename| format!("{}::{}", enum_record_name, rename),
                );

            // Tuple structs have no field names, but instead use the tuple indexes
            let field_idents = variant
                .fields
                .clone()
                .into_iter()
                .enumerate()
                .map(|(idx, field)| {
                    let field_span = field.span();
                    let field_ident = field.ident.clone().map_or_else(
                        || IdentOrIndex::Index(Index { index: idx as u32, span: field_span }),
                        IdentOrIndex::Ident,
                    );
                    let ident = format_ident!("_{}", field.ident.map_or_else(|| idx.to_string(), |ident| ident.to_string()));

                    quote! {
                        #field_ident: #ident,
                    }
                });

            let field_records = variant
                .fields
                .into_iter()
                .enumerate()
                .map(|(idx, field)| {
                    // Tuple structs have no field names, but instead use the tuple indexes
                    let field_name = field.ident.map_or_else(|| idx.to_string(), |ident| ident.to_string());
                    let field_ident = format_ident!("_{}", field_name);
                    let field_type = field.ty;

                    // If the field is renamed within records then use that as the name to extract
                    let field_record_name = get_rename("IntoRecord", "into_record", field.attrs.iter())?
                        .unwrap_or_else(|| field_name.clone());

                    // Call `FromRecord::from_record()` directly on each field
                    Ok(quote! {
                        (
                            std::borrow::Cow::Borrowed(#field_record_name),
                            <#field_type as differential_datalog::record::IntoRecord>::into_record(#field_ident),
                        ),
                    })
                })
                .collect::<Result<TokenStream>>()?;

            // Generate the code for each match arm individually
            Ok(quote! {
                #enum_ident::#variant_ident { #( #field_idents )* } => differential_datalog::record::Record::NamedStruct(
                    std::borrow::Cow::Borrowed(#variant_record_name),
                    vec![#field_records],
                ),
            })
        })
        .collect::<Result<TokenStream>>()?;

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics differential_datalog::record::IntoRecord for #enum_ident #type_generics #where_clause {
            fn into_record(self) -> differential_datalog::record::Record {
                match self {
                    #variants
                }
            }
        }
    })
}
