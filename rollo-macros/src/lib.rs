use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::{parse, parse_macro_input, ItemStruct};

/// Implement WorldTime
#[proc_macro_attribute]
pub fn world_time(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let _ = parse_macro_input!(args as parse::Nothing);
    let name = item_struct.clone().ident;

    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        fields.named.push(
            syn::Field::parse_named
                .parse2(quote! { elapsed: AtomicI64 })
                .unwrap(),
        );
    }

    return quote! {
        use std::sync::atomic::{AtomicI64, Ordering};
        use rollo::server::world::WorldTime;
        #item_struct

        impl WorldTime for #name {
            fn time(&self) -> i64 {
                self.elapsed.load(Ordering::Acquire)
            }

            fn update_time(&self, new_time: i64) {
                self.elapsed.store(new_time, Ordering::Release);
            }
        }
    }
    .into();
}
