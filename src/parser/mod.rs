use pest::Parser;
use pest_derive::Parser;

pub mod ast;
mod clause;
mod expression;
mod statement;
mod evaluator;

#[derive(Parser)]
#[grammar = "parser/sql.pest"]
struct SqlParser;

pub fn parse(input: &str) -> Result<ast::Statement, String> {
    let pairs = SqlParser::parse(Rule::statement, input)
        .map_err(|e| e.to_string())?;

    let stmt_pair = pairs
        .into_iter()
        .next()
        .ok_or_else(|| "empty input".to_string())?;

    Ok(statement::parse_statement(stmt_pair))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::*;

    #[test]
    fn test_select_all() {
        let sql = "SELECT * FROM users;";
        let result = parse(sql).expect("should parse successfully");

        assert_eq!(
            result,
            Statement::Select {
                columns: SelectColumns::All,
                from: "users".to_string(),
                where_clause: None,
                order_by: None,
            }
        );
    }

    #[test]
    fn test_select_with_columns_and_alias() {
        let sql = "SELECT id, name AS \"n\" FROM users WHERE age >= 18 AND active = 1 ORDER BY name;";
        let result = parse(sql).expect("should parse successfully");

        match result {
            Statement::Select { columns, from, where_clause, order_by } => {
                assert_eq!(from, "users");

                match columns {
                    SelectColumns::Expressions(items) => {
                        assert_eq!(items.len(), 2);

                        assert_eq!(items[0].expr, Expr::Identifier("id".to_string()));
                        assert_eq!(items[0].alias, None);

                        assert_eq!(items[1].expr, Expr::Identifier("name".to_string()));
                        assert_eq!(items[1].alias, Some("n".to_string()));
                    }
                    _ => panic!("Expected Expressions variant"),
                }

                assert!(where_clause.is_some());
                let where_expr = where_clause.unwrap();
                match where_expr {
                    Expr::BinaryOp { left, op, right } => {
                        assert_eq!(op, BinOp::And);

                        match *left {
                            Expr::BinaryOp { ref left, ref op, ref right } => {
                                assert_eq!(**left, Expr::Identifier("age".to_string()));
                                assert_eq!(*op, BinOp::Gte);
                                assert_eq!(**right, Expr::Number(18));
                            }
                            _ => panic!("Expected BinaryOp for left side of AND"),
                        }

                        match *right {
                            Expr::BinaryOp { ref left, ref op, ref right } => {
                                assert_eq!(**left, Expr::Identifier("active".to_string()));
                                assert_eq!(*op, BinOp::Eq);
                                assert_eq!(**right, Expr::Number(1));
                            }
                            _ => panic!("Expected BinaryOp for right side of AND"),
                        }
                    }
                    _ => panic!("Expected BinaryOp for WHERE clause"),
                }

                assert!(order_by.is_some());
                assert_eq!(order_by.unwrap().column, "name");
            }
            _ => panic!("Expected Select statement"),
        }
    }

    #[test]
    fn test_create_table() {
        let sql = "CREATE TABLE users (id INTEGER, name VARCHAR(255));";
        let result = parse(sql).expect("should parse successfully");

        assert_eq!(
            result,
            Statement::CreateTable {
                name: "users".to_string(),
                columns: vec![
                    ColumnDef {
                        name: "id".to_string(),
                        data_type: DataType::Integer,
                    },
                    ColumnDef {
                        name: "name".to_string(),
                        data_type: DataType::Varchar(255),
                    },
                ],
            }
        );
    }

    #[test]
    fn test_insert_into() {
        let sql = "INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob');";
        let result = parse(sql).expect("should parse successfully");

        assert_eq!(
            result,
            Statement::Insert {
                table: "users".to_string(),
                values: vec![
                    vec![
                        Expr::Number(1),
                        Expr::StringLit("Alice".to_string()),
                    ],
                    vec![
                        Expr::Number(2),
                        Expr::StringLit("Bob".to_string()),
                    ],
                ],
            }
        );
    }

    #[test]
    fn test_expression_precedence() {
        let sql = "SELECT a + b * c FROM t;";
        let result = parse(sql).expect("should parse successfully");

        match result {
            Statement::Select { columns, from, .. } => {
                assert_eq!(from, "t");

                match columns {
                    SelectColumns::Expressions(items) => {
                        assert_eq!(items.len(), 1);

                        match &items[0].expr {
                            Expr::BinaryOp { left, op, right } => {
                                assert_eq!(*op, BinOp::Add);
                                assert_eq!(**left, Expr::Identifier("a".to_string()));

                                match &**right {
                                    Expr::BinaryOp { left, op, right } => {
                                        assert_eq!(*op, BinOp::Mul);
                                        assert_eq!(**left, Expr::Identifier("b".to_string()));
                                        assert_eq!(**right, Expr::Identifier("c".to_string()));
                                    }
                                    _ => panic!("Expected BinaryOp for b * c"),
                                }
                            }
                            _ => panic!("Expected BinaryOp for a + b * c"),
                        }
                    }
                    _ => panic!("Expected Expressions variant"),
                }
            }
            _ => panic!("Expected Select statement"),
        }
    }

    #[test]
    fn test_unary_minus() {
        let sql = "SELECT -x + 1 FROM t;";
        let result = parse(sql).expect("should parse successfully");

        match result {
            Statement::Select { columns, from, .. } => {
                assert_eq!(from, "t");

                match columns {
                    SelectColumns::Expressions(items) => {
                        assert_eq!(items.len(), 1);

                        match &items[0].expr {
                            Expr::BinaryOp { left, op, right } => {
                                assert_eq!(*op, BinOp::Add);

                                match &**left {
                                    Expr::UnaryOp { op, expr } => {
                                        assert_eq!(*op, UnaryOp::Neg);
                                        assert_eq!(**expr, Expr::Identifier("x".to_string()));
                                    }
                                    _ => panic!("Expected UnaryOp for -x"),
                                }

                                assert_eq!(**right, Expr::Number(1));
                            }
                            _ => panic!("Expected BinaryOp for -x + 1"),
                        }
                    }
                    _ => panic!("Expected Expressions variant"),
                }
            }
            _ => panic!("Expected Select statement"),
        }
    }

    #[test]
    fn test_not_and_or_operators() {
        let sql = "SELECT * FROM t WHERE NOT a = 1 OR b > 2;";
        let result = parse(sql).expect("should parse successfully");

        match result {
            Statement::Select { columns, from, where_clause, order_by } => {
                assert_eq!(columns, SelectColumns::All);
                assert_eq!(from, "t");
                assert!(order_by.is_none());

                let where_expr = where_clause.expect("should have WHERE clause");
                match where_expr {
                    Expr::BinaryOp { left, op, right } => {
                        assert_eq!(op, BinOp::Or);

                        match *left {
                            Expr::UnaryOp { op, expr } => {
                                assert_eq!(op, UnaryOp::Not);

                                match *expr {
                                    Expr::BinaryOp { left, op, right } => {
                                        assert_eq!(op, BinOp::Eq);
                                        assert_eq!(*left, Expr::Identifier("a".to_string()));
                                        assert_eq!(*right, Expr::Number(1));
                                    }
                                    _ => panic!("Expected BinaryOp inside NOT"),
                                }
                            }
                            _ => panic!("Expected UnaryOp for NOT"),
                        }

                        match *right {
                            Expr::BinaryOp { left, op, right } => {
                                assert_eq!(op, BinOp::Gt);
                                assert_eq!(*left, Expr::Identifier("b".to_string()));
                                assert_eq!(*right, Expr::Number(2));
                            }
                            _ => panic!("Expected BinaryOp for b > 2"),
                        }
                    }
                    _ => panic!("Expected BinaryOp for WHERE clause"),
                }
            }
            _ => panic!("Expected Select statement"),
        }
    }

    #[test]
    fn test_all_comparison_operators() {
        let test_cases = vec![
            ("SELECT * FROM t WHERE a = 1;", BinOp::Eq),
            ("SELECT * FROM t WHERE a != 1;", BinOp::Neq),
            ("SELECT * FROM t WHERE a <> 1;", BinOp::Neq),
            ("SELECT * FROM t WHERE a < 1;", BinOp::Lt),
            ("SELECT * FROM t WHERE a > 1;", BinOp::Gt),
            ("SELECT * FROM t WHERE a <= 1;", BinOp::Lte),
            ("SELECT * FROM t WHERE a >= 1;", BinOp::Gte),
        ];

        for (sql, expected_op) in test_cases {
            let result = parse(sql).expect("should parse successfully");
            match result {
                Statement::Select { where_clause, .. } => {
                    match where_clause.expect("should have WHERE clause") {
                        Expr::BinaryOp { op, .. } => {
                            assert_eq!(op, expected_op, "Failed for SQL: {}", sql);
                        }
                        _ => panic!("Expected BinaryOp for: {}", sql),
                    }
                }
                _ => panic!("Expected Select statement for: {}", sql),
            }
        }
    }

    #[test]
    fn test_arithmetic_operators() {
        let test_cases = vec![
            ("SELECT a + b FROM t;", BinOp::Add),
            ("SELECT a - b FROM t;", BinOp::Sub),
            ("SELECT a * b FROM t;", BinOp::Mul),
            ("SELECT a / b FROM t;", BinOp::Div),
        ];

        for (sql, expected_op) in test_cases {
            let result = parse(sql).expect("should parse successfully");
            match result {
                Statement::Select { columns, .. } => {
                    match columns {
                        SelectColumns::Expressions(items) => {
                            match &items[0].expr {
                                Expr::BinaryOp { op, .. } => {
                                    assert_eq!(*op, expected_op, "Failed for SQL: {}", sql);
                                }
                                _ => panic!("Expected BinaryOp for: {}", sql),
                            }
                        }
                        _ => panic!("Expected Expressions for: {}", sql),
                    }
                }
                _ => panic!("Expected Select statement for: {}", sql),
            }
        }
    }

    #[test]
    fn test_string_literals() {
        let sql = "SELECT 'hello world' FROM t;";
        let result = parse(sql).expect("should parse successfully");

        match result {
            Statement::Select { columns, .. } => {
                match columns {
                    SelectColumns::Expressions(items) => {
                        assert_eq!(items[0].expr, Expr::StringLit("hello world".to_string()));
                    }
                    _ => panic!("Expected Expressions variant"),
                }
            }
            _ => panic!("Expected Select statement"),
        }
    }

    #[test]
    fn test_multiple_inserts() {
        let sql = "INSERT INTO t VALUES (1, 'a'), (2, 'b'), (3, 'c');";
        let result = parse(sql).expect("should parse successfully");

        match result {
            Statement::Insert { table, values } => {
                assert_eq!(table, "t");
                assert_eq!(values.len(), 3);

                assert_eq!(values[0], vec![Expr::Number(1), Expr::StringLit("a".to_string())]);
                assert_eq!(values[1], vec![Expr::Number(2), Expr::StringLit("b".to_string())]);
                assert_eq!(values[2], vec![Expr::Number(3), Expr::StringLit("c".to_string())]);
            }
            _ => panic!("Expected Insert statement"),
        }
    }

    #[test]
    fn test_multiple_columns_create_table() {
        let sql = "CREATE TABLE test (id INTEGER, name VARCHAR(100), age INTEGER);";
        let result = parse(sql).expect("should parse successfully");

        match result {
            Statement::CreateTable { name, columns } => {
                assert_eq!(name, "test");
                assert_eq!(columns.len(), 3);

                assert_eq!(columns[0].name, "id");
                assert_eq!(columns[0].data_type, DataType::Integer);

                assert_eq!(columns[1].name, "name");
                assert_eq!(columns[1].data_type, DataType::Varchar(100));

                assert_eq!(columns[2].name, "age");
                assert_eq!(columns[2].data_type, DataType::Integer);
            }
            _ => panic!("Expected CreateTable statement"),
        }
    }
}
