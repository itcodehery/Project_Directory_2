use crate::file_system_state::FileSystemState;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use sqlparser::ast::{Statement, Query, SetExpr, TableFactor, SelectItem, Expr, BinaryOperator, Value};
use comfy_table::{Table, Cell, Color as CColor, Attribute};
use comfy_table::presets::UTF8_FULL;
use std::fs;
use std::os::unix::fs::MetadataExt;
use chrono::{DateTime, Local};

pub fn execute_sql_query(sys_state: &mut FileSystemState, query: &str) -> Result<String, String> {
    let dialect = GenericDialect {};
    let ast = Parser::parse_sql(&dialect, query).map_err(|e| format!("SQL Parse Error: {}", e))?;

    if ast.is_empty() {
        return Err("Empty SQL query".to_string());
    }

    match &ast[0] {
        Statement::Query(q) => execute_select(sys_state, q),
        _ => Err("Unsupported SQL statement. Only SELECT is supported currently.".to_string()),
    }
}

fn execute_select(sys_state: &mut FileSystemState, query: &Query) -> Result<String, String> {
    if let SetExpr::Select(select) = &*query.body {
        // Check FROM clause
        if select.from.is_empty() {
            return Err("Missing FROM clause".to_string());
        }

        let from = &select.from[0].relation;
        let target_dir = match from {
            TableFactor::Table { name, .. } => name.to_string(),
            _ => return Err("Unsupported FROM clause".to_string()),
        };

        if target_dir != "." && target_dir != "files" {
            return Err(format!("Unsupported table '{}'. Use '.' or 'files' to query the current directory.", target_dir));
        }

        let current_path = sys_state.get_current_path();
        
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec![
            Cell::new("Name").fg(CColor::Green).add_attribute(Attribute::Bold),
            Cell::new("Ext").fg(CColor::Green).add_attribute(Attribute::Bold),
            Cell::new("Size").fg(CColor::Green).add_attribute(Attribute::Bold),
            Cell::new("Modified").fg(CColor::Green).add_attribute(Attribute::Bold),
            Cell::new("Is Dir").fg(CColor::Green).add_attribute(Attribute::Bold),
        ]);

        // Basic traversal
        for entry in walkdir::WalkDir::new(current_path).min_depth(1).max_depth(1) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let ext = path.extension().unwrap_or_default().to_string_lossy().to_string();
            let metadata = entry.metadata().ok();
            
            let is_dir = path.is_dir();
            
            let size = if let Some(m) = &metadata {
                m.len()
            } else {
                0
            };

            let modified = if let Some(m) = &metadata {
                if let Ok(sys_time) = m.modified() {
                    let datetime: DateTime<Local> = sys_time.into();
                    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                } else {
                    "-".to_string()
                }
            } else {
                "-".to_string()
            };

            // Implement WHERE clause filtering
            if let Some(selection) = &select.selection {
                if !evaluate_expr(selection, &name, &ext, size, is_dir) {
                    continue;
                }
            }

            table.add_row(vec![
                Cell::new(name).fg(if is_dir { CColor::Green } else { CColor::White }),
                Cell::new(ext),
                Cell::new(size.to_string()),
                Cell::new(modified),
                Cell::new(is_dir.to_string()),
            ]);
        }

        crate::cprintln!("\n{}", table);
        Ok(format!("Executed SQL query"))
    } else {
        Err("Unsupported query type".to_string())
    }
}

fn evaluate_expr(expr: &Expr, name: &str, ext: &str, size: u64, is_dir: bool) -> bool {
    match expr {
        Expr::BinaryOp { left, op, right } => {
            match op {
                BinaryOperator::And => evaluate_expr(left, name, ext, size, is_dir) && evaluate_expr(right, name, ext, size, is_dir),
                BinaryOperator::Or => evaluate_expr(left, name, ext, size, is_dir) || evaluate_expr(right, name, ext, size, is_dir),
                BinaryOperator::Eq => {
                    let (l_val, r_val) = get_values(left, right, name, ext, size, is_dir);
                    l_val == r_val
                },
                BinaryOperator::NotEq => {
                    let (l_val, r_val) = get_values(left, right, name, ext, size, is_dir);
                    l_val != r_val
                },
                BinaryOperator::Gt => {
                    let (l_val, r_val) = get_values(left, right, name, ext, size, is_dir);
                    l_val > r_val
                },
                BinaryOperator::Lt => {
                    let (l_val, r_val) = get_values(left, right, name, ext, size, is_dir);
                    l_val < r_val
                },
                BinaryOperator::GtEq => {
                    let (l_val, r_val) = get_values(left, right, name, ext, size, is_dir);
                    l_val >= r_val
                },
                BinaryOperator::LtEq => {
                    let (l_val, r_val) = get_values(left, right, name, ext, size, is_dir);
                    l_val <= r_val
                },
                _ => false,
            }
        }
        Expr::Nested(nested) => evaluate_expr(nested, name, ext, size, is_dir),
        _ => false,
    }
}

// A helper to resolve values for comparison. For simplicity, we convert everything to Strings,
// except if both look like numbers, we could convert to f64. To handle `size > 100`, we need numeric comparison.
#[derive(PartialEq, PartialOrd)]
enum TypedValue {
    Number(f64),
    Text(String),
    Boolean(bool),
    Null,
}

fn get_values(left: &Expr, right: &Expr, name: &str, ext: &str, size: u64, is_dir: bool) -> (TypedValue, TypedValue) {
    let l = eval_value(left, name, ext, size, is_dir);
    let r = eval_value(right, name, ext, size, is_dir);
    
    // If one is Number and the other is Text that parses to Number, cast it.
    match (&l, &r) {
        (TypedValue::Number(_), TypedValue::Text(s)) => {
            if let Ok(n) = s.parse::<f64>() {
                return (l, TypedValue::Number(n));
            }
        }
        (TypedValue::Text(s), TypedValue::Number(_)) => {
            if let Ok(n) = s.parse::<f64>() {
                return (TypedValue::Number(n), r);
            }
        }
        _ => {}
    }
    
    (l, r)
}

fn eval_value(expr: &Expr, name: &str, ext: &str, size: u64, is_dir: bool) -> TypedValue {
    match expr {
        Expr::Identifier(ident) => {
            let col = ident.value.to_lowercase();
            match col.as_str() {
                "name" => TypedValue::Text(name.to_string()),
                "ext" => TypedValue::Text(ext.to_string()),
                "size" => TypedValue::Number(size as f64),
                "is_dir" => TypedValue::Boolean(is_dir),
                _ => TypedValue::Null,
            }
        }
        Expr::Value(val) => {
            match &**val {
                Value::Number(n, _) => {
                    if let Ok(num) = n.parse::<f64>() {
                        TypedValue::Number(num)
                    } else {
                        TypedValue::Text(n.clone())
                    }
                }
                Value::SingleQuotedString(s) | Value::DoubleQuotedString(s) => TypedValue::Text(s.clone()),
                Value::Boolean(b) => TypedValue::Boolean(*b),
                _ => TypedValue::Null,
            }
        }
        _ => TypedValue::Null,
    }
}
