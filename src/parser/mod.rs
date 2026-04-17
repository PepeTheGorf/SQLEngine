use std::fmt;
use pest::Parser;
use pest_derive::Parser;
use pest::error::Error as PestError;

pub mod ast;
pub mod evaluator;
pub mod clause;
pub mod expression;
pub mod statement;
pub mod binder;

#[derive(Parser)]
#[grammar = "parser/sql.pest"]
struct SqlParser;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub input_line: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "error: {}", self.message)?;
        writeln!(f, "  --> line {}:{}", self.line, self.col)?;
        writeln!(f, "   |")?;
        writeln!(f, " {} | {}", self.line, self.input_line)?;
        write!(f,   "   | {}^", " ".repeat(self.col.saturating_sub(1)))
    }
}

impl std::error::Error for ParseError {}

pub fn parse(input: &str) -> Result<ast::Statement, ParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ParseError {
            message: "empty input — expected a SQL statement".into(),
            line: 1,
            col: 1,
            input_line: String::new(),
        });
    }

    match SqlParser::parse(Rule::statement, input) {
        Ok(pairs) => {
            let stmt_pair = pairs.into_iter().next().unwrap();
            Ok(statement::parse_statement(stmt_pair))
        }
        Err(pest_err) => Err(diagnose(input, pest_err)),
    }
}

fn pest_line_col(err: &PestError<Rule>) -> (usize, usize) {
    match err.line_col {
        pest::error::LineColLocation::Pos((l, c)) => (l, c),
        pest::error::LineColLocation::Span((l, c), _) => (l, c),
    }
}

fn get_source_line(input: &str, line: usize) -> String {
    input.lines().nth(line.saturating_sub(1)).unwrap_or("").to_string()
}

fn diagnose(input: &str, pest_err: PestError<Rule>) -> ParseError {
    let (line, col) = pest_line_col(&pest_err);
    let src = get_source_line(input, line);
    let upper = input.to_uppercase();
    let tokens: Vec<&str> = upper.split_whitespace().collect();
    // Keep a "raw" token stream for quoting in diagnostics, but normalize common
    // punctuation so tests and messages stay stable (e.g. "ta;" -> "ta").
    let raw_tokens: Vec<String> = input
        .split_whitespace()
        .map(|t| t.trim_end_matches(';').to_string())
        .collect();

    let err = |msg: String, c: usize| ParseError {
        message: msg,
        line,
        col: c,
        input_line: src.clone(),
    };

    let token_col = |idx: usize| -> usize {
        let mut pos = 0;
        for (i, tok) in src.split_whitespace().enumerate() {
            if let Some(found) = src[pos..].find(tok) {
                if i == idx {
                    return pos + found + 1;
                }
                pos += found + tok.len();
            }
        }
        col
    };

    if tokens.first() == Some(&"CREATE") {
        if tokens.len() < 2 {
            return err("incomplete statement. expected TABLE after CREATE".into(), col);
        }
        if tokens[1] != "TABLE" {
            let found = raw_tokens.get(1).map(|s| s.as_str()).unwrap_or("");
            return err(
                format!("expected keyword TABLE after CREATE, found \"{}\"", found),
                token_col(1),
            );
        }
        if tokens.len() < 3 {
            return err("expected a table name after CREATE TABLE".into(), col);
        }
        if !src.contains('(') {
            return err(
                "expected column definitions in parentheses.".into(),
                col,
            );
        }
        if !src.trim().ends_with(';') {
            let end = src.len() + 1;
            return err("missing semicolon `;` at end of CREATE TABLE statement".into(), end);
        }
        return err(
            "invalid CREATE TABLE syntax. Expected: CREATE TABLE <name> (<column> <type>, ...);".into(),
            col,
        );
    }

    if tokens.first() == Some(&"INSERT") {
        if tokens.len() < 2 {
            return err("incomplete statement — expected INTO after INSERT".into(), col);
        }
        if tokens[1] != "INTO" {
            return err(
                format!("expected keyword INTO after INSERT, found \"{}\"", raw_tokens[1]),
                token_col(1),
            );
        }
        if tokens.len() < 3 {
            return err("expected a table name after INSERT INTO".into(), col);
        }
        if !upper.contains("VALUES") {
            return err("missing VALUES keyword. Expected: INSERT INTO <table> VALUES (...);".into(), col);
        }
        if !src.contains('(') {
            return err("expected value list in parentheses after VALUES".into(), col);
        }
        if !src.trim().ends_with(';') {
            let end = src.len() + 1;
            return err("missing semicolon `;` at end of INSERT statement".into(), end);
        }
        return err(
            "invalid INSERT syntax. Expected: INSERT INTO <table> VALUES (<val>, ...), ...;".into(),
            col,
        );
    }

    if tokens.first() == Some(&"SELECT") {
        if tokens.len() < 2 {
            return err("incomplete statement — expected column list or * after SELECT".into(), col);
        }
        if !upper.contains("FROM") {
            return err("missing FROM clause. Expected: SELECT <columns> FROM <table>;".into(), col);
        }
        let from_pos = tokens.iter().position(|t| *t == "FROM");
        if let Some(pos) = from_pos {
            if pos + 1 >= tokens.len() {
                return err("expected a table name after FROM".into(), col);
            }
        }
        if !src.trim().ends_with(';') {
            let end = src.len() + 1;
            return err("missing semicolon `;` at end of SELECT statement".into(), end);
        }
        return err(
            "invalid SELECT syntax. Expected: SELECT <columns> FROM <table> [WHERE ...] [ORDER BY ...];".into(),
            col,
        );
    }

    let first = raw_tokens.first().map(|s| s.as_str()).unwrap_or("");
    err(
        format!(
            "unrecognized statement starting with \"{}\". Expected one of: CREATE TABLE, INSERT INTO, SELECT",
            first
        ),
        1,
    )
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

    // ---- Error reporting tests ----

    #[test]
    fn test_error_create_missing_table_keyword() {
        let err = parse("CREATE ta;").unwrap_err();
        assert!(err.message.contains("expected keyword TABLE after CREATE"));
        assert!(err.message.contains("\"ta\""));
    }

    #[test]
    fn test_error_create_incomplete() {
        let err = parse("CREATE").unwrap_err();
        assert!(err.message.contains("expected TABLE after CREATE"));
    }

    #[test]
    fn test_error_create_missing_semicolon() {
        let err = parse("CREATE TABLE users (id INTEGER)").unwrap_err();
        assert!(err.message.contains("missing semicolon"));
    }

    #[test]
    fn test_error_create_missing_parens() {
        let err = parse("CREATE TABLE users;").unwrap_err();
        assert!(err.message.contains("parentheses"));
    }

    #[test]
    fn test_error_insert_missing_into() {
        let err = parse("INSERT users VALUES (1);").unwrap_err();
        assert!(err.message.contains("expected keyword INTO after INSERT"));
        assert!(err.message.contains("\"users\""));
    }

    #[test]
    fn test_error_insert_missing_values() {
        let err = parse("INSERT INTO users (1);").unwrap_err();
        assert!(err.message.contains("VALUES"));
    }

    #[test]
    fn test_error_insert_missing_semicolon() {
        let err = parse("INSERT INTO users VALUES (1)").unwrap_err();
        assert!(err.message.contains("missing semicolon"));
    }

    #[test]
    fn test_error_select_missing_from() {
        let err = parse("SELECT *;").unwrap_err();
        assert!(err.message.contains("FROM"));
    }

    #[test]
    fn test_error_select_missing_semicolon() {
        let err = parse("SELECT * FROM users").unwrap_err();
        assert!(err.message.contains("missing semicolon"));
    }

    #[test]
    fn test_error_unknown_statement() {
        let err = parse("DROP TABLE users;").unwrap_err();
        assert!(err.message.contains("unrecognized statement"));
        assert!(err.message.contains("\"DROP\""));
    }

    #[test]
    fn test_error_empty_input() {
        let err = parse("").unwrap_err();
        assert!(err.message.contains("empty input"));
    }

    #[test]
    fn test_error_display_format() {
        let err = parse("CREATE ta;").unwrap_err();
        let output = format!("{}", err);
        // Should contain the rustc-style error format
        assert!(output.contains("error:"));
        assert!(output.contains("-->"));
        assert!(output.contains("|"));
        assert!(output.contains("^"));
        assert!(output.contains("CREATE ta;"));
    }
}
