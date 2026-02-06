extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, LitStr};

#[proc_macro_derive(Insertable, attributes(table_name))]
pub fn insertable_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    // Get table_name from #[table_name = "tasks"] attribute
    let mut table_name = struct_name.to_string().to_lowercase();
    for attr in input.attrs.iter() {
        if attr.path().is_ident("table_name") {
            if let syn::Meta::NameValue(nv) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &nv.value {
                    if let syn::Lit::Str(litstr) = &expr_lit.lit {
                        table_name = litstr.value();
                    }
                }
            }
        }
    }

    // Collect field names
    let field_idents: Vec<_> = match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Named(ref fields_named) => fields_named.named.iter().map(|f| f.ident.as_ref().unwrap()).collect(),
            _ => panic!("Insertable only works on structs with named fields"),
        },
        _ => panic!("Insertable only works on structs"),
    };

    let columns: Vec<LitStr> = field_idents.iter()
        .map(|f| LitStr::new(&f.to_string(), f.span()))
        .collect();
    let values = field_idents.iter();

    let expanded = quote! {
        impl Insertable for #struct_name {
            fn table_name() -> &'static str {
                #table_name
            }

            fn columns_values(&self) -> Vec<(&'static str, &dyn rusqlite::ToSql)> {
                vec![
                    #(
                        (#columns, &self.#values as &dyn rusqlite::ToSql),
                    )*
                ]
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Queryable)]
pub fn queryable_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    // Collect field names
    let field_idents: Vec<_> = match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Named(ref fields_named) => fields_named.named.iter().map(|f| f.ident.as_ref().unwrap()).collect(),
            _ => panic!("Queryable only works on structs with named fields"),
        },
        _ => panic!("Queryable only works on structs"),
    };

    let field_count = field_idents.len();
    let indices: Vec<usize> = (0..field_count).collect();

    let expanded = quote! {
        impl #struct_name {
            pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
                Ok(Self {
                    #(
                        #field_idents: row.get(#indices)?,
                    )*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Updatable, attributes(table_name))]
pub fn updatable_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    // Get table_name from #[table_name = "tasks"] attribute
    let mut table_name = struct_name.to_string().to_lowercase();
    for attr in input.attrs.iter() {
        if attr.path().is_ident("table_name") {
            if let syn::Meta::NameValue(nv) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &nv.value {
                    if let syn::Lit::Str(litstr) = &expr_lit.lit {
                        table_name = litstr.value();
                    }
                }
            }
        }
    }

    // Collect field names and types
    let fields: Vec<_> = match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Named(ref fields_named) => {
                fields_named.named.iter().map(|f| {
                    let ident = f.ident.as_ref().unwrap();
                    let ty = &f.ty;
                    (ident, ty)
                }).collect()
            },
            _ => panic!("Updatable only works on structs with named fields"),
        },
        _ => panic!("Updatable only works on structs"),
    };

    let field_pushes = fields.iter().map(|(field_ident, field_ty)| {
        let field_name = LitStr::new(&field_ident.to_string(), field_ident.span());
        
        // Check if the type is Option<Option<T>> (nested option for nullable fields)  
        let ty_str = quote!(#field_ty).to_string();
        let is_option = ty_str.starts_with("Option");
        let is_nested_option = is_option && ty_str.matches("Option").count() >= 2;
        
        if is_nested_option {
            // Handle Option<Option<T>> - for nullable fields
            quote! {
                if let Some(ref val) = self.#field_ident {
                    match val {
                        Some(v) => cols.push((#field_name, v as &dyn rusqlite::ToSql)),
                        None => cols.push((#field_name, &rusqlite::types::Null as &dyn rusqlite::ToSql)),
                    }
                }
            }
        } else if is_option {
            // Handle regular Option<T>
            quote! {
                if let Some(ref val) = self.#field_ident {
                    cols.push((#field_name, val as &dyn rusqlite::ToSql));
                }
            }
        } else {
            // Handle direct field (non-Option) - always include
            quote! {
                cols.push((#field_name, &self.#field_ident as &dyn rusqlite::ToSql));
            }
        }
    });

    let expanded = quote! {
        impl crate::db::Updatable for #struct_name {
            fn table_name() -> &'static str {
                #table_name
            }

            fn update_columns_values(&self) -> Vec<(&'static str, &dyn rusqlite::ToSql)> {
                let mut cols: Vec<(&'static str, &dyn rusqlite::ToSql)> = Vec::new();
                
                #(
                    #field_pushes
                )*
                
                cols
            }
        }
    };

    TokenStream::from(expanded)
}
