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
                .parse2(quote! { time: std::sync::atomic::AtomicI64 })
                .unwrap(),
        );
    }

    let tokens = quote! {
        #item_struct
        use rollo::server::world::WorldTime;

        impl WorldTime for #name {
            fn time(&self) -> i64 {
                self.time.load(std::sync::atomic::Ordering::Acquire)
            }

            fn update_time(&self, new_time: i64) {
                self.time.store(new_time, std::sync::atomic::Ordering::Release);
            }
        }
    };

    tokens.into()
}
