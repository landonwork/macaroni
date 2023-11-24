use sorted_macro::sorted;

#[sorted]
enum TestEnum {
    Variant1,
    Variant2
}

#[test]
fn main() {
    let _enum1 = TestEnum::Variant1;
    let _enum2 = TestEnum::Variant2;
}
