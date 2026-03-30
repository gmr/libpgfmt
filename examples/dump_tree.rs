use tree_sitter::Parser;
use tree_sitter_postgres::LANGUAGE;

fn print_tree(node: tree_sitter::Node, source: &str, indent: usize) {
    let kind = node.kind();
    let text = &source[node.byte_range()];
    let short = if text.len() > 80 { &text[..80] } else { text };
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
    let path = std::env::args()
        .nth(1)
        .expect("usage: dump_tree <file.sql>");
    let sql = std::fs::read_to_string(&path).unwrap();
    let input = if sql.trim().ends_with(';') {
        sql.trim().to_string()
    } else {
        format!("{};", sql.trim())
    };
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    let tree = parser.parse(&input, None).unwrap();
    print_tree(tree.root_node(), &input, 0);
}
