//! Plain-text and CSV tables for reports.

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub title: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    pub fn add_row(&mut self, cells: &[String]) -> Result<(), String> {
        if (cells.len() as i64) != (self.headers.len() as i64) {
            return Err(format!(
                "row has {} cells, table {} has {} columns",
                cells.len() as i64,
                self.title,
                self.headers.len() as i64
            ));
        }
        self.rows.push(cells.to_vec());
        Ok(())
    }

    pub fn widths(&self) -> Vec<i64> {
        let mut widths: Vec<i64> = Vec::new();
        for header in self.headers.iter() {
            widths.push(header.len() as i64);
        }
        for row in self.rows.iter() {
            for i in 0..row.len() as i64 {
                if (row[i as usize].len() as i64) > widths[i as usize] {
                    widths[i as usize] = row[i as usize].len() as i64;
                }
            }
        }
        widths
    }

    pub fn render(&self) -> String {
        let widths = self.widths();
        let mut lines: Vec<String> = vec![self.title.clone()];
        let mut head: Vec<String> = Vec::new();
        for i in 0..self.headers.len() as i64 {
            head.push(format!(
                "{}{}",
                self.headers[i as usize],
                " ".repeat((widths[i as usize] as usize).saturating_sub(self.headers[i as usize].len()))
            ));
        }
        lines.push(head.join(" | "));
        for row in self.rows.iter() {
            let mut padded: Vec<String> = Vec::new();
            for i in 0..row.len() as i64 {
                padded.push(format!(
                    "{}{}",
                    row[i as usize],
                    " ".repeat((widths[i as usize] as usize).saturating_sub(row[i as usize].len()))
                ));
            }
            lines.push(padded.join(" | "));
        }
        lines.join("\n")
    }

    pub fn csv(&self) -> String {
        let mut lines: Vec<String> = vec![self.headers.join(",")];
        for row in self.rows.iter() {
            lines.push(row.join(","));
        }
        lines.join("\n")
    }
}

pub fn new_table(title: &str, headers: &[String]) -> Table {
    Table {
        title: title.to_string(),
        headers: headers.to_vec(),
        rows: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_rows_of_the_wrong_width() {
        let mut grid = new_table("Totals", &vec!["region".to_string(), "total".to_string()]);
        assert!(
            grid.add_row(&vec!["north".to_string()]).is_err(),
            "expected short row to fail"
        );
        if grid.add_row(&vec!["north".to_string(), "12".to_string()]).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(grid.csv(), "region,total\nnorth,12", "csv");
    }
}
