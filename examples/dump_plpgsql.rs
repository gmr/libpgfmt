use tree_sitter::Parser;
use tree_sitter_postgres::LANGUAGE_PLPGSQL;

fn print_tree(node: tree_sitter::Node, source: &str, indent: usize) {
    let kind = node.kind();
    let text = &source[node.byte_range()];
    let short = if text.len() > 60 { &text[..60] } else { text };
    let short = short.replace('\n', "\\n");
    let pad = "  ".repeat(indent);
    if node.is_named() {
        println!("{pad}{kind}: {short:?}");
    } else {
        println!("{pad}[{kind}]: {short:?}");
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_tree(child, source, indent + 1);
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("usage: dump_plpgsql <file>");
    let sql = std::fs::read_to_string(&path).unwrap();
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE_PLPGSQL.into()).unwrap();
    let tree = parser.parse(sql.trim(), None).unwrap();
    print_tree(tree.root_node(), sql.trim(), 0);
}
