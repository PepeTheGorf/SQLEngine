use crate::parser::ast::Expr;
use crate::parser::evaluator::Evaluator;

mod error;
mod parser;
mod executor;
mod storage;

fn main() {
    println!("Enter SQL commands (type 'exit' to quit):");
    let mut executor = executor::Executor::new();

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") {
            break;
        }

        if input.eq_ignore_ascii_case("!print") {
            println!("Current tables in memory:");
            for (table_name, table) in &executor.context.tables {
                println!("Table: {}", table_name);
                println!("Columns:");
                for column in &table.columns {
                    println!("  - {} ({:?})", column.name, column.data_type);
                }
                println!("Rows:");
                for row in &table.rows {
                    println!("  - {:?}", row.values);
                }
            }
            continue;
        }

        match parser::parse(input) {
            Ok(statement) => {
                match executor.execute(statement) {
                    Ok(result) => println!("Execution result: {:?}", result),
                    Err(e) => eprintln!("Execution error: {}", e),
                }
            }
            Err(e) => eprintln!("Parse error: {}", e),
        }
    }
}
