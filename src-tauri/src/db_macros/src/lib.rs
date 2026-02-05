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
