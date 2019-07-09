fn main() {
    let mut conf = lalrpop::Configuration::new();
    let conf = conf.use_cargo_dir_conventions();

    conf.process_file("src/cesar_parser.lalrpop").unwrap();
    conf.process_file("src/bnf_parser.lalrpop").unwrap();

    println!("cargo:rerun-if-changed=src/cesar_parser.lalrpop");
    println!("cargo:rerun-if-changed=src/bnf_parser.lalrpop");
}
