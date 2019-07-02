fn main() {
    lalrpop::Configuration::new()
        .use_cargo_dir_conventions()
        .process_file("src/cesar.lalrpop")
        .unwrap();

    println!("cargo:rerun-if-changed=src/cesar.lalrpop");
}
