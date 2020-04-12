fn main() {
    let mut lalrpop_conf = lalrpop::Configuration::new();
    lalrpop_conf.use_cargo_dir_conventions().emit_rerun_directives(true).emit_report(true);

    lalrpop_conf.process_file("src/ascesis_parser.lalrpop").unwrap();
    lalrpop_conf.process_file("src/bnf_parser.lalrpop").unwrap();
}
