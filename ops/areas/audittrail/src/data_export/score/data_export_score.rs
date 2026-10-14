use super::*;

use acme_kit::text;

/// Where a data export score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportScoreStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DataExportScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DataExportScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export score as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportScore {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub retries: i64,
    pub ceiling: f64,
    pub contact_email: String,
    pub margin: f64,
    pub namespace: String,
    pub currency: String,
    pub locale: String,
    pub status: DataExportScoreStatus,
}

impl DataExportScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportScoreStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportScoreStatus) -> bool {
        match self.status {
            DataExportScoreStatus::Created => {
                match next {
                    DataExportScoreStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportScoreStatus::Validated => {
                match next {
                    DataExportScoreStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportScoreStatus::Published => {
                return next == DataExportScoreStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "data export score", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.score - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportScore {
        DataExportScore {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            retries: self.retries,
            ceiling: self.ceiling,
            contact_email: self.contact_email.clone(),
            margin: self.margin,
            namespace: self.namespace.clone(),
            currency: self.currency.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a data export score in status created.
pub fn new_data_export_score(code: &str, label: &str) -> DataExportScore {
    DataExportScore {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        retries: 0,
        ceiling: 0.0,
        contact_email: "".to_string(),
        margin: 0.0,
        namespace: "".to_string(),
        currency: "".to_string(),
        locale: "".to_string(),
        status: DataExportScoreStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_data_export_score("code-684", "label-804");
        assert_eq!(item.status, DataExportScoreStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_data_export_score("code-684", "label-804");
        if item.move_to(DataExportScoreStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportScoreStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_data_export_score("code-684", "label-804");
        item.status = DataExportScoreStatus::Retired;
        assert!(
            item.move_to(DataExportScoreStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
