extern crate proc_macro;
extern crate proc_macro2;
extern crate proc_macro_crate;
extern crate quote;
extern crate syn;

mod parse;

use parse::{Data, Fields, TestGeneratorInput, TransmittableInput};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{parse_macro_input, Error, Result};

fn get_crate_name() -> Ident {
    let ident = match crate_name("transmittable").expect("transmittable is present in `Cargo.toml`") {
        FoundCrate::Itself => "crate".to_string(),
        FoundCrate::Name(name) => name.clone(),
    };

    Ident::new(ident.as_str(), Span::call_site())
}

#[proc_macro]
pub fn read_and_write(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TestGeneratorInput);
    let crate_name = get_crate_name();
    let ty = input.ty;
    let ident = input.base_name;

    let read_fn  = Ident::new(&format!("read_{}",  ident), ident.span());
    let write_fn = Ident::new(&format!("write_{}", ident), ident.span());
    let cases = input.cases.iter()
        .map(|case| {
            let serialized = case.serialized.clone();
            let deserialized = case.deserialized.clone();
            quote!((#serialized, #deserialized))
        })
        .collect::<Vec<_>>();

    TokenStream::from(quote! {
        #[test]
        fn #read_fn() {
            let cases: [(&[u8], #crate_name::Result<#ty>); _] = [#(#cases),*];
            let processed_cases = cases.iter()
                .map(|(bytes, expected)| (std::io::Cursor::new(bytes), expected))
                .collect::<Vec<_>>();

            for (mut bytes, expected) in processed_cases {
                let res: #crate_name::Result<#ty> = #crate_name::Transmittable::deserialize(&mut bytes);
                println!("Deserialized {:?}: {:?}", bytes.get_ref(), res);

                assert!(match (expected, res) {
                    (Ok(v1),  Ok(v2))  => *v1 == v2,
                    (Err(e1), Err(e2)) => *e1 == e2,
                    _ => false,
                });

                let has_data = (bytes.get_ref().len() - bytes.position() as usize) > 0; // std::io::BufRead::has_data_left(&mut bytes)
                assert!(!has_data, "Function did not read the whole buffer");
            }
        }

        #[test]
        fn #write_fn() {
            let cases: [(&[u8], #crate_name::Result<#ty>); _] = [#(#cases),*];
            let processed_cases = cases.iter()
                .flat_map(|(bytes, expected)| expected.as_ref().ok().map(|v| (*bytes, v.clone())))
                .collect::<Vec<_>>();

            for (expected, value) in processed_cases {
                let mut bytes = std::io::Cursor::new(Vec::with_capacity(expected.len()));
                #crate_name::Transmittable::serialize(&value, &mut bytes).expect("Failed to serialize the value");
                let inner = bytes.into_inner();
                println!("Serialized {:?}: {:?}", value, inner);

                assert_eq!(inner, expected);
            }
        }
    })
}


#[proc_macro_derive(Transmittable)]
pub fn transmittable_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TransmittableInput);
    let crate_name = get_crate_name();

    TokenStream::from((match input.data {
        // We can ignore the data, as we pass the whole input itself to the impl function
        Data::Struct(_) => impl_struct(input, crate_name),
        Data::Enum(_) => impl_enum(input, crate_name),
        _ => Err(Error::new(Span::call_site(), "Only structs and enums are supported")),
    }).unwrap_or_else(|e| e.to_compile_error()))
}

fn impl_struct(input: TransmittableInput, crate_name: Ident) -> Result<TokenStream2> {
    let Data::Struct(fields) = input.data else {
        return Err(Error::new(Span::call_site(), "Expected a struct"));
    };

    let ident = input.ident;

    match fields {
        Fields::Unnamed(count) => {
            let iter = (0..count).map(|i| syn::Index::from(i as usize));
            let deserialize = iter.clone().map(|_| quote!( #crate_name::Transmittable::deserialize(reader)? ));

            Ok(quote! {
                impl #crate_name::Transmittable for #ident {
                    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> #crate_name::Result<()> {
                        #( #crate_name::Transmittable::serialize(&self.#iter, writer)?; )*
                        Ok(())
                    }

                    fn deserialize<R: std::io::Read>(reader: &mut R) -> #crate_name::Result<Self> {
                        Ok(Self(#(#deserialize),*))
                    }
                }
            })
        },
        Fields::Named(named) => {
            let idents = named.iter();
            let deserialize = idents.clone().map(|ident| quote!( #ident: #crate_name::Transmittable::deserialize(reader)? ));

            Ok(quote! {
                impl #crate_name::Transmittable for #ident {
                    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> #crate_name::Result<()> {
                        #( #crate_name::Transmittable::serialize(&self.#idents, writer)?; )*
                        Ok(())
                    }

                    fn deserialize<R: std::io::Read>(reader: &mut R) -> #crate_name::Result<Self> {
                        Ok(Self {
                            #( #deserialize ),*
                        })
                    }
                }
            })
        },
        Fields::Empty => Err(Error::new(Span::call_site(), "Expected a struct with fields")),
    }
}

fn impl_enum(input: TransmittableInput, crate_name: Ident) -> Result<TokenStream2> {
    let Data::Enum(variants) = input.data else {
        return Err(Error::new(Span::call_site(), "Expected an enum."));
    };

    let Some(repr) = input.repr else {
        return Err(Error::new(Span::call_site(), "Enums without a repr are not supported."));
    };

    let get_discriminant = {
        #[cfg(feature = "unsafe")]
        quote! { unsafe { *<*const _>::from(self).cast::<#repr>() } }
        #[cfg(not(feature = "unsafe"))]
        quote! {
            match self {
                _ => unreachable!()
            }
        }
    };

    let identifier = input.ident;

    let serialize_arms = variants.iter()
        .filter_map(|variant| {
            let ident = &variant.ident;

            match variant.fields.clone() {
                Fields::Unnamed(count) => {
                    let iter = 0..count;
                    let variables = iter.clone().map(|i| Ident::new(&format!("var{}", i), Span::call_site()));
                    let serialize = variables.clone().map(|var| quote!( #crate_name::Transmittable::serialize(#var, writer)?; ));

                    Some(quote! {
                        #identifier::#ident(#(#variables),*) => {
                            #(#serialize)*
                        }
                    })
                },
                Fields::Named(names) => Some(quote! {
                    #identifier::#ident {#(#names),*} => {
                        #(#crate_name::Transmittable::serialize(#names, writer)?;)*
                    }
                }),
                Fields::Empty => None,
            }
        })
        .collect::<Vec<TokenStream2>>();

    let deserialize_arms = variants.iter()
        .map(|variant| {
            let ident = &variant.ident;

            let body = match variant.fields.clone() {
                Fields::Unnamed(count) => {
                    let deserialize = (0..count).map(|_| quote!(#crate_name::Transmittable::deserialize(reader)?));

                    quote!(#identifier::#ident(#(#deserialize),*))
                },
                Fields::Named(names) => quote!(#identifier::#ident {
                    #(#names: #crate_name::Transmittable::deserialize(reader)?),*
                }),
                Fields::Empty => quote!(#identifier::#ident),
            };

            quote!(discriminants::#ident => Ok(#body))
        });

    let struct_impl = variants.iter()
        .map(|variant| {
            let ident = &variant.ident;
            let discrim = &variant.discriminant;
            quote!(const #ident: #repr = #discrim;)
        });

    Ok(quote! {
        impl #crate_name::Transmittable for #identifier {
            fn serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
                #crate_name::Transmittable::serialize(&#get_discriminant, writer)?;

                match self {
                    #(#serialize_arms,)*
                    _ => ()
                }

                Ok(())
            }

            fn deserialize<R: std::io::Read>(reader: &mut R) -> Result<Self> {
                let discriminant: #repr = #crate_name::Transmittable::deserialize(reader)?;

                struct discriminants;

                #[allow(non_upper_case_globals)]
                impl discriminants {
                    #(#struct_impl)*
                }

                match discriminant {
                    #(#deserialize_arms,)*
                    _ => Err(#crate_name::Error::InvalidEnumVariant),
                }
            }
        }
    })
}