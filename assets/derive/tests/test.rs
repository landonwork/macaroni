use my_macro::HelloMacro;

pub trait HelloMacro {
    fn hello_macro();
}

#[derive(HelloMacro)]
struct Macaroni;

fn main() {
    Macaroni::hello_macro()
}
