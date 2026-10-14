use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a data export rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DataExportRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for DataExportRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRule {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub currency: String,
    pub quantity: i64,
    pub attributes: BTreeMap<String, String>,
    pub grace_days: i64,
    pub approved: bool,
    pub status: DataExportRuleStatus,
}

impl DataExportRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportRuleStatus) -> bool {
        match self.status {
            DataExportRuleStatus::Planned => {
                match next {
                    DataExportRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportRuleStatus::Scheduled => {
                return next == DataExportRuleStatus::Cancelled;
            }
            DataExportRuleStatus::InProgress => {
                match next {
                    DataExportRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportRuleStatus::Completed => {
                return next == DataExportRuleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "data export rule", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn projected_load(&self) -> f64 {
        ((self.grace_days as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportRule {
        DataExportRule {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            currency: self.currency.clone(),
            quantity: self.quantity,
            attributes: self.attributes.clone(),
            grace_days: self.grace_days,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a data export rule in status planned.
pub fn new_data_export_rule(code: &str, name: &str) -> DataExportRule {
    DataExportRule {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        currency: "".to_string(),
        quantity: 0,
        attributes: BTreeMap::new(),
        grace_days: 0,
        approved: false,
        status: DataExportRuleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_data_export_rule("code-476", "name-727");
        assert_eq!(item.status, DataExportRuleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_data_export_rule("code-476", "name-727");
        if item.move_to(DataExportRuleStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportRuleStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_data_export_rule("code-476", "name-727");
        item.status = DataExportRuleStatus::Cancelled;
        assert!(
            item.move_to(DataExportRuleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
