use std::collections::HashMap;
use comfy_table::{Table, Cell, Color as CColor, Attribute, TableComponent};
use comfy_table::presets::UTF8_FULL;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Record(HashMap<String, Value>),
    Table(Vec<HashMap<String, Value>>),
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Null,
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Null => "null".to_string(),
            Value::Record(map) => {
                let mut table = Table::new();
                table.load_preset(comfy_table::presets::UTF8_FULL);
                table.set_style(TableComponent::VerticalLines, '│');
                table.set_style(TableComponent::MiddleIntersections, '┼');
                table.set_style(TableComponent::HorizontalLines, '─');
                table.set_style(TableComponent::MiddleHeaderIntersections, '┼');
                table.set_style(TableComponent::HeaderLines, '─');
                table.set_style(TableComponent::LeftHeaderIntersection, '├');
                table.set_style(TableComponent::RightHeaderIntersection, '┤');
                table.set_header(vec!["Key", "Value"]);
                for (k, v) in map {
                    table.add_row(vec![k.clone(), v.to_string()]);
                }
                table.to_string()
            }
            Value::Table(rows) => {
                if rows.is_empty() {
                    return "(empty table)".to_string();
                }
                let mut table = Table::new();
                table.load_preset(comfy_table::presets::UTF8_FULL);
                table.set_style(TableComponent::VerticalLines, '│');
                table.set_style(TableComponent::MiddleIntersections, '┼');
                table.set_style(TableComponent::HorizontalLines, '─');
                table.set_style(TableComponent::MiddleHeaderIntersections, '┼');
                table.set_style(TableComponent::HeaderLines, '─');
                table.set_style(TableComponent::LeftHeaderIntersection, '├');
                table.set_style(TableComponent::RightHeaderIntersection, '┤');
                
                // Get all unique keys
                let mut keys: Vec<String> = rows.iter()
                    .flat_map(|r| r.keys().cloned())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();
                keys.sort(); // Sort keys for consistent columns
                
                let mut header_cells = Vec::new();
                for k in &keys {
                    header_cells.push(Cell::new(k).fg(CColor::Green).add_attribute(Attribute::Bold));
                }
                table.set_header(header_cells);
                
                for row in rows {
                    let mut row_cells = Vec::new();
                    for k in &keys {
                        if let Some(val) = row.get(k) {
                            row_cells.push(Cell::new(val.to_string()));
                        } else {
                            row_cells.push(Cell::new(""));
                        }
                    }
                    table.add_row(row_cells);
                }
                table.to_string()
            }
        }
    }
}
