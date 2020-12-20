use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Data, DataEnum, DataStruct, DeriveInput, Error, GenericParam,
    Generics, Ident, ImplGenerics, Lit, MetaNameValue, Result, TypeGenerics, TypeParamBound,
    WhereClause,
};

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
///     bar: usize,
/// }
/// ```
///
#[proc_macro_derive(FromRecord, attributes(ddlog, from_record))]
pub fn derive_from_record(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    from_record_inner(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn from_record_inner(input: DeriveInput) -> Result<TokenStream> {
    // If `#[ddlog(rename = "...")]` or `#[ddlog(from_record = "")] is provided, the struct will
    // be renamed to the given string for its `FromRecord` implementation
    let struct_string = input
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("ddlog"))
        .map(|attr| attr.parse_args::<MetaNameValue>())
        .find(|attr| {
            attr.as_ref()
                .map(|attr| attr.path.is_ident("rename") || attr.path.is_ident("from_record"))
                .unwrap_or_default()
        })
        .transpose()?
        .map(|meta| {
            if let Lit::Str(string) = meta.lit {
                Ok(string.value())
            } else {
                Err(Error::new_spanned(
                    meta.lit,
                    "`FromRecord` can only be renamed to string literal values",
                ))
            }
        })
        .transpose()?
        .unwrap_or_else(|| input.ident.to_string());
    let struct_name = input.ident;

    // Add the required trait bounds
    let generics = add_trait_bounds(
        input.generics,
        vec![
            parse_quote!(differential_datalog::record::FromRecord),
            parse_quote!(Sized),
            parse_quote!(std::default::Default),
            parse_quote!(serde::de::DeserializeOwned),
        ],
    );
    let generics = generics.split_for_impl();

    match input.data {
        Data::Struct(derive_struct) => {
            from_record_struct(struct_name, struct_string, derive_struct, generics)
        }

        Data::Enum(derive_enum) => {
            from_record_enum(struct_name, struct_string, derive_enum, generics)
        }

        Data::Union(union) => Err(Error::new_spanned(
            union.union_token,
            "`FromRecord` is not able to be automatically implemented on unions",
        )),
    }
}

fn from_record_struct(
    struct_name: Ident,
    struct_string: String,
    derive_struct: DataStruct,
    (impl_generics, type_generics, where_clause): (
        ImplGenerics,
        TypeGenerics,
        Option<&WhereClause>,
    ),
) -> Result<TokenStream> {
    let num_fields = derive_struct.fields.len();

    // The innards of `FromRecord` for positional structs
    let positional_fields: TokenStream = derive_struct
        .fields.iter()
        .cloned()
        .enumerate()
        .map(|(idx, field)| {
            // Tuple structs have no field names, but instead use the tuple indexes
            let field_name = field.ident.unwrap_or_else(|| parse_quote!(#idx));
            let field_type = field.ty;

            quote! {
                #field_name: <#field_type as differential_datalog::record::FromRecord>::from_record(&args[#idx])?,
            }
        })
        .collect();

    // The innards of `FromRecord` for named structs
    let named_fields = derive_struct
        .fields
        .into_iter()
        .enumerate()
        .map(|(idx, field)| {
            // Tuple structs have no field names, but instead use the tuple indexes
            let field_name = field.ident.unwrap_or_else(|| parse_quote!(#idx));
            let field_type = field.ty;

            // If the field is renamed within records then use that as the name to extract
            let record_name = field
                .attrs
                .iter()
                .filter(|attr| attr.path.is_ident("ddlog"))
                .map(|attr| attr.parse_args::<MetaNameValue>())
                .find(|attr| {
                    attr.as_ref()
                        .map(|attr| attr.path.is_ident("rename") || attr.path.is_ident("from_record"))
                        .unwrap_or_default()
                })
                .transpose()?
                .map(|meta| {
                    if let Lit::Str(string) = meta.lit {
                        Ok(string.value())
                    } else {
                        Err(Error::new_spanned(
                            meta.lit,
                            "`FromRecord` fields can only be renamed to string literal values",
                        ))
                    }
                })
                .transpose()?
                .unwrap_or_else(|| field_name.to_string());

            Ok(quote! {
                #field_name: differential_datalog::record::arg_extract::<#field_type>(args, #record_name)?,
            })
        })
        .collect::<Result<TokenStream>>()?;

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics differential_datalog::record::FromRecord for #struct_name #type_generics #where_clause {
            fn from_record(record: &differential_datalog::record::Record) -> std::result::Result<Self, std::string::String> {
                match record {
                    differential_datalog::record::Record::PosStruct(constructor, args) => {
                        match constructor.as_ref() {
                            #struct_string if args.len() == #num_fields => {
                                std::result::Result::Ok(Self { #positional_fields })
                            },

                            error => {
                                std::result::Result::Err(format!(
                                    "unknown constructor {} of type '{}' in {:?}",
                                    error, #struct_string, *record,
                                ))
                            }
                        }
                    },

                    differential_datalog::record::Record::NamedStruct(constructor, args) => {
                        match constructor.as_ref() {
                            #struct_string => {
                                std::result::Result::Ok(Self { #named_fields })
                            },

                            error => {
                                std::result::Result::Err(format!(
                                    "unknown constructor {} of type '{}' in {:?}",
                                    error, #struct_string, *record,
                                ))
                            }
                        }
                    },

                    error => {
                        std::result::Result::Err(format!("not a struct {:?}", *error))
                    },
                }
            }
        }
    })
}

fn from_record_enum(
    enum_name: Ident,
    enum_string: String,
    derive_enum: DataEnum,
    (impl_generics, type_generics, where_clause): (
        ImplGenerics,
        TypeGenerics,
        Option<&WhereClause>,
    ),
) -> Result<TokenStream> {
    let positional_variants = derive_enum
        .variants
        .iter()
        .cloned()
        .map(|variant| {
            let num_fields = variant.fields.len();
            let variant_name = variant.ident;

            // If the variant is renamed within records then use that as the name to extract
            let record_name = variant
                .attrs
                .iter()
                .filter(|attr| attr.path.is_ident("ddlog"))
                .map(|attr| attr.parse_args::<MetaNameValue>())
                .find(|attr| {
                    attr.as_ref()
                        .map(|attr| attr.path.is_ident("rename") || attr.path.is_ident("from_record"))
                        .unwrap_or_default()
                })
                .transpose()?
                .map(|meta| {
                    if let Lit::Str(string) = meta.lit {
                        Ok(string.value())
                    } else {
                        Err(Error::new_spanned(
                            meta.lit,
                            "`FromRecord` variants can only be renamed to string literal values",
                        ))
                    }
                })
                .transpose()?
                .unwrap_or_else(|| variant_name.to_string());

            let positional_fields: TokenStream = variant
                .fields
                .iter()
                .cloned()
                .enumerate()
                .map(|(idx, field)| {
                    // Tuple structs have no field names, but instead use the tuple indexes
                    let field_name = field.ident.unwrap_or_else(|| parse_quote!(idx));
                    let field_type = field.ty;

                    quote! {
                        #field_name: <#field_type as differential_datalog::record::FromRecord>::from_record(&args[#idx])?,
                    }
                })
                .collect();

            Ok(quote! {
                #record_name if args.len() == #num_fields => {
                    std::result::Result::Ok(Self::#variant_name { #positional_fields })
                },
            })
        })
        .collect::<Result<TokenStream>>()?;

    let named_variants = derive_enum
        .variants
        .into_iter()
        .map(|variant| {
            let num_fields = variant.fields.len();
            let variant_name = variant.ident;

            // If the variant is renamed within records then use that as the name to extract
            let record_name = variant
                .attrs
                .iter()
                .filter(|attr| attr.path.is_ident("ddlog"))
                .map(|attr| attr.parse_args::<MetaNameValue>())
                .find(|attr| {
                    attr.as_ref()
                        .map(|attr| attr.path.is_ident("rename") || attr.path.is_ident("from_record"))
                        .unwrap_or_default()
                })
                .transpose()?
                .map(|meta| {
                    if let Lit::Str(string) = meta.lit {
                        Ok(string.value())
                    } else {
                        Err(Error::new_spanned(
                            meta.lit,
                            "`FromRecord` variants can only be renamed to string literal values",
                        ))
                    }
                })
                .transpose()?
                .unwrap_or_else(|| variant_name.to_string());

            let named_fields = variant
                .fields
                .into_iter()
                .enumerate()
                .map(|(idx, field)| {
                    // Tuple structs have no field names, but instead use the tuple indexes
                    let field_name = field.ident.unwrap_or_else(|| parse_quote!(#idx));
                    let field_type = field.ty;

                    // If the field is renamed within records then use that as the name to extract
                    let record_name = field
                        .attrs
                        .iter()
                        .filter(|attr| attr.path.is_ident("ddlog"))
                        .map(|attr| attr.parse_args::<MetaNameValue>())
                        .find(|attr| {
                            attr.as_ref()
                                .map(|attr| attr.path.is_ident("rename") || attr.path.is_ident("from_record"))
                                .unwrap_or_default()
                        })
                        .transpose()?
                        .map(|meta| {
                            if let Lit::Str(string) = meta.lit {
                                Ok(string.value())
                            } else {
                                Err(Error::new_spanned(
                                    meta.lit,
                                    "`FromRecord` fields can only be renamed to string literal values",
                                ))
                            }
                        })
                        .transpose()?
                        .unwrap_or_else(|| field_name.to_string());

                    Ok(quote! {
                        #field_name: differential_datalog::record::arg_extract::<#field_type>(args, #record_name)?,
                    })
                })
                .collect::<Result<TokenStream>>()?;

            Ok(quote! {
                #record_name if args.len() == #num_fields => {
                    std::result::Result::Ok(Self::#variant_name { #named_fields })
                },
            })
        })
        .collect::<Result<TokenStream>>()?;

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics differential_datalog::record::FromRecord for #enum_name #type_generics #where_clause {
            fn from_record(record: &differential_datalog::record::Record) -> std::result::Result<Self, String> {
                match record {
                    differential_datalog::record::Record::PosStruct(constructor, args) => {
                        match constructor.as_ref() {
                            #positional_variants

                            error => {
                                std::result::Result::Err(format!(
                                    "unknown constructor {} of type '{}' in {:?}",
                                    error, #enum_string, *record,
                                ))
                            },
                        }
                    },

                    differential_datalog::record::Record::NamedStruct(constructor, args) => {
                        match constructor.as_ref() {
                            #named_variants

                            error => {
                                std::result::Result::Err(format!(
                                    "unknown constructor {} of type '{}' in {:?}",
                                    error, #enum_string, *record,
                                ))
                            }
                        }
                    },

                    error => {
                        std::result::Result::Err(format!("not a struct {:?}", *error))
                    }
                }
            }
        }
    })
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
