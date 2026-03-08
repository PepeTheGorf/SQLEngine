mod error;
mod parser;
mod executor;
mod storage;

fn main() {
    let examples = [
        "SELECT * FROM users;",
        "SELECT id, name AS \"n\" FROM users WHERE age >= 18 AND active = 1 ORDER BY name;",
        "CREATE TABLE users (id INTEGER, name VARCHAR(255));",
        "INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob');",
        "SELECT a + b * c FROM t;",            // tests precedence
        "SELECT -x + 1 FROM t;",               // tests unary minus
        "SELECT * FROM t WHERE NOT a = 1 OR b > 2;", // tests NOT and OR
    ];

    for sql in &examples {
        println!("SQL: {sql}");
        match parser::parse(sql) {
            Ok(stmt) => {
                println!("AST: {stmt:#?}\n");
            },
            Err(e) => println!("ERR: {e}\n"),
        }
    }
}
