use std::process::Command;

#[test]
fn test_command() {
    Command::new("cargo")
        .arg("expand")
        .arg("--test")
        .arg("test")
        .output()
        .unwrap();
}
