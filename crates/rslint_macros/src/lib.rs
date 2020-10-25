use proc_macro::TokenStream;
use quote::*;
use syn::*;

#[proc_macro_derive(Mergeable)]
pub fn mergeable(input: TokenStream) -> TokenStream {
    let strukt: ItemStruct = parse_macro_input! { input };
    let mut methods = vec![];
    for field in strukt.fields.clone() {
        let name = field.ident.unwrap();
        let tokens = quote! {
            let #name = most_frequent(items.iter().map(|x| x.#name.to_owned()).collect::<Vec<_>>()).to_owned();
        };
        methods.push(tokens);
    }
    let field = strukt.fields.iter().map(|x| x.ident.as_ref().unwrap());
    let name = strukt.ident;
    let tokens = quote! {
        impl #name {
            /// Take multiple instances of the struct and merge them into a single struct which
            /// uses the most common values for each field
            pub fn merge(items: Vec<Self>) -> Option<Self> {
                use crate::util::most_frequent;

                if items.is_empty() {
                    return None;
                }

                #(#methods)*
                Some(Self {
                    #(#field),*
                })
            }
        }
    };
    tokens.into()
}
