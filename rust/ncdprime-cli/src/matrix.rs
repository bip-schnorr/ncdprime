use crate::inputs::InputSet;

pub fn format_matrix(
    rows: &[String],
    cols: &[String],
    values: &[Vec<f64>],
    format: &str,
    labels: bool,
) -> String {
    let sep = if format == "csv" { ',' } else { '\t' };

    let mut out = String::new();

    if labels {
        out.push(sep);
        out.push_str(&cols.join(&sep.to_string()));
        out.push('\n');

        for (r, row_name) in rows.iter().enumerate() {
            out.push_str(row_name);
            for v in &values[r] {
                out.push(sep);
                out.push_str(&v.to_string());
            }
            out.push('\n');
        }
    } else {
        for r in 0..rows.len() {
            for (j, v) in values[r].iter().enumerate() {
                if j > 0 {
                    out.push(sep);
                }
                out.push_str(&v.to_string());
            }
            out.push('\n');
        }
    }

    out
}

pub fn rows_cols(a: &InputSet, b: &InputSet) -> (Vec<String>, Vec<String>) {
    let rows = a.items.iter().map(|i| i.label.clone()).collect();
    let cols = b.items.iter().map(|i| i.label.clone()).collect();
    (rows, cols)
}
