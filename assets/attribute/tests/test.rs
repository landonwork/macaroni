use my_macro::sorted;

#[sorted]
enum TestEnum {
    Variant1,
    Variant2,
}

fn main() {
    let _enum1 = TestEnum::Variant1;
    let _enum2 = TestEnum::Variant2;
}
