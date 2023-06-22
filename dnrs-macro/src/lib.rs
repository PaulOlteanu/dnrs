use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parser, parse_macro_input, parse_quote, DeriveInput};

#[proc_macro_attribute]
pub fn resource_record(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);

    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            match &mut struct_data.fields {
                syn::Fields::Named(fields) => {
                    let new_fields = [
                        quote! {pub name: Name},
                        quote! {pub type_: RecordType},
                        quote! {
                            pub class: u16
                        },
                        quote! {
                            #[derivative(Hash = "ignore", PartialEq = "ignore")]
                            pub ttl: u32
                        },
                    ]
                    .into_iter()
                    .map(|x| syn::Field::parse_named.parse2(x).unwrap());

                    fields.named.extend(new_fields);
                }
                _ => (),
            }

            ast.attrs.push(parse_quote! {#[derive(Derivative)]});
            ast.attrs
                .push(parse_quote! {#[derivative(Debug, Clone, Hash, PartialEq, Eq)]});

            let name = &ast.ident;

            let imp: syn::ItemImpl = parse_quote! {
                impl ByteSer for #name {
                    fn to_bytes(&self) -> Bytes {
                        let mut ret = BytesMut::new();
                        ret.extend_from_slice(&self.name.to_bytes());
                        ret.put_u16(self.type_.to_int());
                        ret.extend_from_slice(&self.class.to_be_bytes());
                        ret.extend_from_slice(&self.ttl.to_be_bytes());
                        let data = self.data_to_bytes();
                        let rd_length = data.len() as u16;
                        ret.extend_from_slice(&rd_length.to_be_bytes());
                        ret.extend_from_slice(&data);

                        ret.into()
                    }
                }
            };

            return quote! {
                #ast

                #imp
            }
            .into();
        }
        _ => panic!("`add_field` has to be used with structs "),
    }
}
