use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

// ─── HasId ────────────────────────────────────────────────────────────────────
//
// Generates:
//   impl HasId for T {
//       fn get_id(&self) -> String { self.<field>.clone() }
//   }
//
// Defaults to the field named `id`. Override with #[has_id(field = "other")].
//
// The `HasId` trait must be in scope at the call site (e.g. from webapp-lib).
//
// Example:
//   #[derive(HasId)]
//   pub struct User { pub id: String, pub display_name: String }
//
//   #[derive(HasId)]
//   #[has_id(field = "email")]
//   pub struct UserRequest { pub email: String, pub display_name: String }

#[proc_macro_derive(HasId, attributes(has_id))]
pub fn derive_has_id(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let field_name = find_attribute_field(&input.attrs, "has_id")
        .unwrap_or_else(|| "id".to_string());
    let field = syn::Ident::new(&field_name, Span::call_site());

    TokenStream::from(quote! {
        impl HasId for #name {
            fn get_id(&self) -> String {
                self.#field.clone()
            }
        }
    })
}

// ─── HasName ──────────────────────────────────────────────────────────────────
//
// Generates:
//   impl HasName for T {
//       fn get_name(&self) -> String { self.<field>.clone() }
//   }
//
// Defaults to the field named `name`. Override with #[has_name(field = "other")].
//
// The `HasName` trait must be in scope at the call site (e.g. from webapp-lib).
//
// Example:
//   #[derive(HasName)]
//   pub struct User { pub id: String, pub name: String }
//
//   #[derive(HasName)]
//   #[has_name(field = "display_name")]
//   pub struct UserAccount { pub id: String, pub display_name: String }

#[proc_macro_derive(HasName, attributes(has_name))]
pub fn derive_has_name(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let field_name = find_attribute_field(&input.attrs, "has_name")
        .unwrap_or_else(|| "name".to_string());
    let field = syn::Ident::new(&field_name, Span::call_site());

    TokenStream::from(quote! {
        impl HasName for #name {
            fn get_name(&self) -> String {
                self.#field.clone()
            }
        }
    })
}

// ─── FormModel ────────────────────────────────────────────────────────────────
//
// Generates a companion `{Struct}FormModel` with every field wrapped in
// `RwSignal<T>`, plus three methods:
//   - `new()`              — default-initialised signals
//   - `from_{snake}()`    — populate signals from an existing entity
//   - `to_{snake}()`      — collect signal values back into the plain struct
//
// Requires `RwSignal` (from Leptos) and `Default` + `Clone` on all field types
// to be in scope at the call site.
//
// Example:
//   #[derive(FormModel)]
//   pub struct Deal {
//       pub deal_number: String,
//       pub volume: f64,
//   }
//
// Generated:
//   pub struct DealFormModel { pub deal_number: RwSignal<String>, pub volume: RwSignal<f64> }
//   impl DealFormModel {
//       pub fn new() -> Self { ... }
//       pub fn from_deal(source: &Deal) -> Self { ... }
//       pub fn to_deal(&self) -> Deal { ... }
//   }

#[proc_macro_derive(FormModel)]
pub fn derive_form_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let form_name = syn::Ident::new(&format!("{}FormModel", name), name.span());

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("FormModel only supports structs with named fields"),
        },
        _ => panic!("FormModel only supports structs"),
    };

    let signal_fields = fields.iter().map(|f| {
        let fname = &f.ident;
        let ftype = &f.ty;
        quote! { pub #fname: RwSignal<#ftype> }
    });

    let new_fields = fields.iter().map(|f| {
        let fname = &f.ident;
        let ftype = &f.ty;
        quote! { #fname: RwSignal::new(<#ftype>::default()) }
    });

    let from_fields = fields.iter().map(|f| {
        let fname = &f.ident;
        quote! { #fname: RwSignal::new(source.#fname.clone()) }
    });

    let to_fields = fields.iter().map(|f| {
        let fname = &f.ident;
        quote! { #fname: self.#fname.get() }
    });

    let from_method =
        syn::Ident::new(&format!("from_{}", to_snake_case(&name.to_string())), name.span());
    let to_method =
        syn::Ident::new(&format!("to_{}", to_snake_case(&name.to_string())), name.span());

    TokenStream::from(quote! {
        pub struct #form_name {
            #(#signal_fields,)*
        }

        impl #form_name {
            pub fn new() -> Self {
                Self { #(#new_fields,)* }
            }

            pub fn #from_method(source: &#name) -> Self {
                Self { #(#from_fields,)* }
            }

            pub fn #to_method(&self) -> #name {
                #name { #(#to_fields,)* }
            }
        }
    })
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Looks for `#[attr_name(field = "...")]` on the struct and returns the value.
fn find_attribute_field(attrs: &[syn::Attribute], attr_name: &str) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident(attr_name) {
            let mut found = None;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("field") {
                    let value = meta.value()?;
                    let s: syn::LitStr = value.parse()?;
                    found = Some(s.value());
                    Ok(())
                } else {
                    Err(meta.error("unknown attribute key — expected `field = \"...\"`"))
                }
            });
            return found;
        }
    }
    None
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}
