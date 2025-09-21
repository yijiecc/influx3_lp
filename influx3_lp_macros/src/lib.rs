//! Macros for serializing a struct to line protocol string.
//!
//! There is only one derive macro: 
//! * `#[derive(Influx3Lp)]`
//! 
//! There are three kind of attribute-like macros defined: 
//! * `#[influx3_lp(table_name = "home")]` which must be applied to struct level
//! * `#[influx3_lp(timestamp)]` which must be applied to field level
//! * `#[influx3_lp(tag)]` which must be applied to field level
//!
//! Combined together, we can write:
//!
//! ```rust
//! #[derive(Influx3Lp)]
//! #[influx3_lp(table_name = "home")]
//! struct SensorData {
//!     pub temp: f32,
//!     pub hum: Option<f64>,
//!     #[influx3_lp(tag)]
//!     pub room: String,
//!     #[influx3_lp(timestamp)]
//!     pub timestamp: Option<i64>,
//! }
//! ```
//!
//! Escape is applied according to [line protocol](https://docs.influxdata.com/influxdb3/core/reference/line-protocol/#special-characters).

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Influx3Lp, attributes(influx3_lp))]
pub fn influx3_lp_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let mut table_name = None;
    let mut fields = Vec::new();
    let mut tags = Vec::new();
    let mut timestamp = None;

    // struct level attributes 
    // #[influx3_lp(table_name = "home")]
    for attr in &input.attrs {
        if attr.path().is_ident("influx3_lp") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("table_name") {
                    let lit: syn::LitStr = meta.value()?.parse()?;
                    table_name = Some(lit.value().escape_table());
                }
                Ok(())
            });
        }
    }

    // field level attributes 
    // #[influx3_lp(tag)]
    // #[influx3_lp(timestamp)]
    if let syn::Data::Struct(data_struct) = &input.data {
        for field in &data_struct.fields {
            let ident = field.ident.as_ref().unwrap();
            let mut is_tag = false;
            let mut is_timestamp = false;

            // parse attributes
            for attr in &field.attrs {
                if attr.path().is_ident("influx3_lp") {
                    let _ = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("timestamp") {
                            is_timestamp = true;
                        } else if meta.path.is_ident("tag") {
                            is_tag = true;
                        }
                        Ok(())
                    });
                }
            }

            if is_tag {
                if let Some(_) = is_option(&field.ty) {
                    let tag_key = ident.to_string().escape_tag_key();
                    tags.push(quote! {
                        if let Some(v) = &self.#ident {
                            parts.push(format!("{}={}", 
                                               #tag_key, 
                                               v.to_string()
                                               .replace(",", "\\,")
                                               .replace(" ", "\\ ")
                                               .replace("=", "\\=")));
                        }
                    });                    
                } else {
                    let tag_key = ident.to_string().escape_tag_key();
                    tags.push(quote! {
                        parts.push(format!("{}={}", 
                                           #tag_key, 
                                           self.#ident.to_string()
                                           .replace(",", "\\,")
                                           .replace(" ", "\\ ")
                                           .replace("=", "\\=")));
                    });
                }
            } else if is_timestamp {
                if is_option(&field.ty).is_some() {
                    timestamp = Some(quote! {
                        let ts = if let Some(v) = self.#ident {
                            v.to_string()
                        } else {
                            String::new()
                        };
                    });
                } else {
                    timestamp = Some(quote! {
                        let ts = self.#ident.to_string();
                    });
                }
            } else {
                if let Some(ty) = is_option(&field.ty) {
                    let field_key = ident.to_string().escape_field_key();
                    fields.push(quote! {
                        if let Some(v) = &self.#ident {
                            fields.push(format!(
                                "{}={}",
                                #field_key,
                                {
                                    if std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<i8>()
                                        || std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<i16>()
                                        || std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<i32>()
                                        || std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<i64>()
                                    {
                                        format!("{}i", v)
                                    } else if std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<u8>()
                                        || std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<u16>()
                                        || std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<u32>()
                                        || std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<u64>() 
                                    {
                                        format!("{}u", v)
                                    } else if std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<String>() 
                                    {
                                        let t = v.to_string();
                                        if t.len() > 64 * 1024 {
                                            panic!("Length of string field value has a limit of 64K");
                                        }
                                        // string field value should be qutoed
                                        let t = t.replace("\\", "\\\\").replace("\"", "\\\"");
                                        format!("\"{}\"", t)
                                    } else {
                                        format!("{}", v)
                                    }
                                }
                            ));
                        }
                    });
                } else {
                    let ty = &field.ty;
                    let field_key = ident.to_string().escape_field_key();

                    fields.push(quote! {
                        fields.push(format!(
                            "{}={}",
                            #field_key,
                            {
                                let v = &self.#ident;
                                if std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<i8>()
                                    || std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<i16>()
                                    || std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<i32>()
                                    || std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<i64>()
                                {
                                    format!("{}i", v)
                                } else if std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<u8>()
                                    || std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<u16>()
                                    || std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<u32>()
                                    || std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<u64>() 
                                {
                                    format!("{}u", v)
                                } else if std::any::TypeId::of::<#ty>() == std::any::TypeId::of::<String>() 
                                {
                                    let t = v.to_string();
                                    if t.len() > 64 * 1024 {
                                        panic!("Length of string field value has a limit of 64K");
                                    }
                                    // string field value should be qutoed
                                    let t = t.replace("\\", "\\\\").replace("\"", "\\\"");
                                    format!("\"{}\"", t)
                                } else {
                                    format!("{}", v)
                                }
                            }
                        ));
                    });
                }
            }
        }
    }

    let table_name = table_name.expect("Missing table_name in #[influx3_lp]");
    if fields.len() == 0 {
        panic!("{} should have at least one field", struct_name.to_string());
    }

    let expanded = if let Some(timestamp_code) = timestamp {
        quote! {
            impl Influx3Lp for #struct_name {
                fn to_lp(&self) -> String {
                    let mut parts: Vec<String> = Vec::new();
                    let mut fields: Vec<String> = Vec::new();

                    #(#tags)*

                    #(#fields)*

                    #timestamp_code

                    let tags_str = if parts.is_empty() {
                        String::new()
                    } else {
                        format!(",{}", parts.join(","))
                    };

                    if ts.len() > 0 {
                        format!(
                            "{}{} {} {}",
                            #table_name,
                            tags_str,
                            fields.join(","),
                            ts
                        )
                    } else {
                        format!(
                            "{}{} {}",
                            #table_name,
                            tags_str,
                            fields.join(","),
                        )
                    }
                }
            }
        }
    } else {
        quote! {
            impl Influx3Lp for #struct_name {
                fn to_lp(&self) -> String {
                    let mut parts: Vec<String> = Vec::new();
                    let mut fields: Vec<String> = Vec::new();

                    #(#tags)*

                    #(#fields)*

                    let tags_str = if parts.is_empty() {
                        String::new()
                    } else {
                        format!(",{}", parts.join(","))
                    };

                    format!(
                        "{}{} {}",
                        #table_name,
                        tags_str,
                        fields.join(","),
                    )
                }
            }
        }
    };


    TokenStream::from(expanded)
}

/// a helper to detect if a field of struct is Option
fn is_option(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(typepath) = ty {
        if typepath.qself.is_none() && typepath.path.segments.len() == 1 {
            let segment = &typepath.path.segments[0];
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}

// Escape string according to line protocol
// https://docs.influxdata.com/influxdb3/core/reference/line-protocol/#special-characters
#[allow(dead_code)]
trait StringExtensions {
    fn escape_table(self) -> String;
    fn escape_tag_key(self) -> String;
    fn escape_tag_value(self) -> String;
    fn escape_field_key(self) -> String;
    fn escape_field_value(self) -> String;
}

impl StringExtensions for String {
    fn escape_table(self) -> String {
        self.replace(",", "\\,").replace(" ", "\\ ")
    }

    fn escape_tag_key(self) -> String {
        self.replace(",", "\\,").replace(" ", "\\ ").replace("=", "\\=")
    }

    fn escape_tag_value(self) -> String {
        self.replace(",", "\\,").replace(" ", "\\ ").replace("=", "\\=")
    }

    fn escape_field_key(self) -> String {
        self.replace(",", "\\,").replace(" ", "\\ ").replace("=", "\\=")
    }

    fn escape_field_value(self) -> String {
        self.replace("\\", "\\\\").replace("\"", "\\\"")
    }
}
