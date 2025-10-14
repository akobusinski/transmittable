use proc_macro2::{Ident, Span};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::BinOp::Add;
use syn::Expr::{Binary, Lit};
use syn::{parenthesized, DeriveInput, Expr, ExprBinary, ExprLit, LitInt, Path, Type};

pub struct TestCase {
    pub serialized: Expr,
    pub deserialized: Expr,
}

pub struct TestGeneratorInput {
    pub ty: Path,
    pub base_name: Ident,
    pub cases: Vec<TestCase>,
}

#[derive(Clone)]
pub struct TransmittableInput {
    pub ident: Ident,
    pub repr: Option<Ident>,
    pub data: Data,
}

#[derive(Clone)]
pub struct Variant {
    pub ident: Ident,
    pub fields: Fields,
    pub discriminant: Expr,
}

#[derive(Clone)]
pub enum Data {
    Struct(Fields),
    Enum(Vec<Variant>),
    Unknown,
}

#[derive(Clone)]
pub enum Fields {
    Empty,
    Unnamed(u32), // the number of fields
    Named(Vec<Ident>), // the names of the fields
}

impl Parse for TestCase {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let serialized = content.parse()?;
        content.parse::<syn::Token![,]>()?;
        let deserialized = content.parse()?;
        Ok(Self {
            serialized,
            deserialized,
        })
    }
}

impl Parse for TestGeneratorInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let Type::Path(type_path) = input.parse()? else {
            return Err(syn::Error::new(input.span(), "Expected a type path"));
        };
        let ty = type_path.path;

        let base_name = ty.segments.last()
            .ok_or_else(|| syn::Error::new(ty.span(), "Expected a type path"))
            .map(|last| last.ident.clone())
            .map(|ident| Ident::new(ident.to_string().to_lowercase().as_str(), ident.span()))?;

        input.parse::<syn::Token![;]>()?;

        let cases = Punctuated::<TestCase, Comma>::parse_terminated(input)?
            .into_iter()
            .collect();

        Ok(Self {
            ty,
            base_name,
            cases,
        })
    }
}

impl Parse for TransmittableInput {
    fn parse(stream: ParseStream) -> syn::Result<Self> {
        let input = DeriveInput::parse(stream)?;
        let repr = match input.data {
            syn::Data::Enum(_) => parse_repr(&input)?, // only check for the repr on enums
            _ => None,
        };

        let data = match input.data {
            syn::Data::Struct(data) => Data::Struct(parse_fields(data.fields)?),
            syn::Data::Enum(data) => {
                let mut current_discriminant = Lit(ExprLit {
                    attrs: Vec::new(),
                    lit: syn::Lit::Int(LitInt::new("0", Span::call_site())),
                });

                Data::Enum(
                    data.variants
                        .iter()
                        .map(|variant| parse_fields(variant.fields.clone())
                            .map(|fields| {
                                let discriminant = match variant.discriminant {
                                    Some((_, ref expr)) => {
                                        current_discriminant = increment_expr(expr.clone());
                                        expr.clone()
                                    }
                                    None => {
                                        let old = current_discriminant.clone();
                                        current_discriminant = increment_expr(old.clone());

                                        old
                                    }
                                };


                                Variant { // we want to keep it as a result so that .collect can return a result as well
                                    ident: variant.ident.clone(),
                                    fields,
                                    discriminant,
                                }
                            })
                        )
                        .collect::<syn::Result<Vec<Variant>>>()?
                )
            },
            _ => Data::Unknown,
        };

        Ok(TransmittableInput {
            ident: input.ident,
            repr,
            data,
        })
    }
}

fn increment_expr(expr: Expr) -> Expr {
    Binary(ExprBinary {
        attrs: Vec::new(),
        left: Box::new(expr.clone()),
        op: Add(Default::default()),
        right: Box::new(Lit(ExprLit {
            attrs: Vec::new(),
            lit: syn::Lit::Int(LitInt::new("1", Span::call_site())),
        })),
    })
}

fn parse_fields(input: syn::Fields) -> syn::Result<Fields> {
    Ok(match input {
        syn::Fields::Named(fields) => Fields::Named(fields.named
            // I chuckled while writing this
            .iter()
            .map(|field| field.ident
                .as_ref()
                .ok_or_else(|| syn::Error::new(field.span(), "named fields must have an identifier"))
                .map(|v| v.to_owned())
            )
            .collect::<syn::Result<Vec<Ident>>>()?
        ),
        syn::Fields::Unnamed(fields) => Fields::Unnamed(fields.unnamed.len() as u32),
        syn::Fields::Unit => Fields::Empty,
    })
}

fn parse_repr(input: &DeriveInput) -> syn::Result<Option<Ident>> {
    let mut found = None;

    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| { // lovely
                if let Some(ident) = meta.path.get_ident() {
                    match ident.to_string().as_str() {
                        | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                        | "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => {
                            found = Some(ident.clone());
                        },
                        _ => ()
                    }
                }

                Ok(())
            })?;

            if found.is_some() {
                return Ok(found);
            }
        }
    }

    Ok(None)
}