const {{ const_name | upper }}: &str = "{{ const_value }}";

fn main() {
    println!("{{ lowercase }}: {}", {{ const_name | upper }});
}
