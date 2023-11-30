use proc_macro::TokenStream;

#[proc_macro]
pub fn my_five_macro(_input: TokenStream) -> TokenStream {
    "5".parse().unwrap()
}
