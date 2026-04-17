use std::path::Path;
use crate::storage::data_structures::Table;

mod error;
mod parser;
mod executor;
mod storage;

fn main() {
    println!("Enter SQL commands (type 'exit' to quit):");
    let mut executor = executor::Executor::new();
    let mut tables = std::collections::HashMap::new();

    let path = Path::new("catalog");

    for json_table in path.read_dir().expect("Failed to read catalog directory") {
        let json_table = json_table.expect("Failed to read table file");
        let table: Table = serde_json::from_reader(
            std::fs::File::open(json_table.path()).expect("Failed to open table file")
        ).expect("Failed to parse table JSON");
        tables.insert(table.name.clone(), table);
    }

    executor.set_context(executor::context::ExecutionContext { tables });
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") {
            break;
        }

        if input.eq_ignore_ascii_case("test-insert") {
            for i in 1..1000000 {
                let salary = rand::random::<u32>() % 49000 + 1000;
                let bonus = rand::random::<u32>() % 10000 + 100;
                let insert_statement = format!("INSERT INTO RadnikTest VALUES ({}, 'User{}', {}, {});", i, i, salary, bonus);

                match parser::parse(&insert_statement) {
                    Ok(statement) => {
                        match executor.execute(statement) {
                            Ok(_) => {}
                            Err(e) => eprintln!("Execution error: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Parse error: {}", e),
                }
            }
            break;
        }

        match parser::parse(input) {
            Ok(statement) => {
                match executor.execute(statement) {
                    Ok(result) => {
                        println!("Execution result: {:?}", result);
                    }
                    Err(e) => eprintln!("Execution error: {}", e),
                }
            }
            Err(e) => eprintln!("Parse error: {}", e),
        }
    }
}
