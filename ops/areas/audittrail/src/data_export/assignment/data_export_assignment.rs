use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a data export assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DataExportAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for DataExportAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export assignment as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAssignment {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub priority: i64,
    pub attributes: BTreeMap<String, String>,
    pub status: DataExportAssignmentStatus,
}

impl DataExportAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportAssignmentStatus) -> bool {
        match self.status {
            DataExportAssignmentStatus::Open => {
                return next == DataExportAssignmentStatus::InReview;
            }
            DataExportAssignmentStatus::InReview => {
                return next == DataExportAssignmentStatus::Reopened || next == DataExportAssignmentStatus::Archived;
            }
            DataExportAssignmentStatus::Resolved => {
                return next == DataExportAssignmentStatus::Reopened || next == DataExportAssignmentStatus::Archived;
            }
            DataExportAssignmentStatus::Reopened => {
                match next {
                    DataExportAssignmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "data export assignment", self.code)
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

    pub fn projected_total(&self) -> f64 {
        ((self.priority as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportAssignment {
        DataExportAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            priority: self.priority,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a data export assignment in status open.
pub fn new_data_export_assignment(code: &str, label: &str) -> DataExportAssignment {
    DataExportAssignment {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        priority: 0,
        attributes: BTreeMap::new(),
        status: DataExportAssignmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_data_export_assignment("code-82", "label-514");
        assert_eq!(item.status, DataExportAssignmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_data_export_assignment("code-82", "label-514");
        if item.move_to(DataExportAssignmentStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportAssignmentStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_data_export_assignment("code-82", "label-514");
        item.status = DataExportAssignmentStatus::Archived;
        assert!(
            item.move_to(DataExportAssignmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
