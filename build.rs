fn main() {
    let mut conf = lalrpop::Configuration::new();
    let conf = conf.use_cargo_dir_conventions();

    conf.process_file("src/cesar_grammar.lalrpop").unwrap();
    conf.process_file("src/bnf_grammar.lalrpop").unwrap();

    println!("cargo:rerun-if-changed=src/cesar_grammar.lalrpop");
    println!("cargo:rerun-if-changed=src/bnf_grammar.lalrpop");
}
