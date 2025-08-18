use sqlparser::{
    ast::{FromTable, Statement, TableFactor},
    dialect::GenericDialect,
    parser::Parser,
};

pub fn extract_span_name_from_sql(sql: &str) -> Option<String> {
    let dialect = GenericDialect {};
    let ast = Parser::parse_sql(&dialect, sql).ok()?;
    let stmt = ast.get(0)?;
    let clean = |name: &str| name.replace(['"', '`'], "");

    match stmt {
        Statement::Query(query) => {
            if let sqlparser::ast::SetExpr::Select(select) = &*query.body {
                if let Some(from) = select.from.get(0) {
                    if let TableFactor::Table { name, .. } = &from.relation {
                        return Some(format!("{} {}", "SELECT".to_string(), clean(&name.to_string())));
                    }
                }
            }
        }
        Statement::Insert(insert) => {
            return Some(format!("{} {}", "INSERT".to_string(), clean(&insert.table.to_string())));
        }
        Statement::Update { table, .. } => {
            return Some(format!("{} {}", "UPDATE".to_string(), clean(&table.to_string())));
        }
        Statement::Delete(delete) => {
            match &delete.from {
                FromTable::WithFromKeyword(from_vec) | FromTable::WithoutKeyword(from_vec) => {
                    if let Some(table_with_joins) = from_vec.get(0) {
                        if let TableFactor::Table { name, .. } = &table_with_joins.relation {
                            // `name` is the ObjectName of the table
                            return Some(format!("{} {}", "DELETE".to_string(), clean(&name.to_string())));
                        }
                    }
                }
            }
        }
        _ => {
            tracing::debug!("Unsupported SQL statement: {:?}", stmt);
            return Some("OTHER".to_string());
        }
    }
    None
}