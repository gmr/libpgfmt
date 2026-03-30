use libpgfmt::{format, style::Style};
fn main() {
    let sql = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let style: Style = std::env::args()
        .nth(2)
        .unwrap_or("aweber".to_string())
        .parse()
        .unwrap();
    match format(sql.trim(), style) {
        Ok(f) => print!("{f}\n"),
        Err(e) => eprintln!("Error: {e}"),
    }
}
