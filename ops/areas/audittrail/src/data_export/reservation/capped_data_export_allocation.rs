use super::*;

use acme_kit::text;

/// Where a capped data export allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedDataExportAllocationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CappedDataExportAllocationStatus {
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

impl std::fmt::Display for CappedDataExportAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped data export allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportAllocation {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub title: String,
    pub budget: f64,
    pub status: CappedDataExportAllocationStatus,
}

impl CappedDataExportAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedDataExportAllocationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedDataExportAllocationStatus) -> bool {
        match self.status {
            CappedDataExportAllocationStatus::Open => {
                return next == CappedDataExportAllocationStatus::Archived;
            }
            CappedDataExportAllocationStatus::InReview => {
                return next == CappedDataExportAllocationStatus::Archived;
            }
            CappedDataExportAllocationStatus::Resolved => {
                return next == CappedDataExportAllocationStatus::Archived;
            }
            CappedDataExportAllocationStatus::Reopened => {
                return next == CappedDataExportAllocationStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedDataExportAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped data export allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped data export allocation", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.priority as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedDataExportAllocation {
        CappedDataExportAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            title: self.title.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped data export allocation in status open.
pub fn new_capped_data_export_allocation(code: &str, name: &str) -> CappedDataExportAllocation {
    CappedDataExportAllocation {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        title: "".to_string(),
        budget: 0.0,
        status: CappedDataExportAllocationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_capped_data_export_allocation("code-949", "name-604");
        assert_eq!(item.status, CappedDataExportAllocationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_capped_data_export_allocation("code-949", "name-604");
        if item.move_to(CappedDataExportAllocationStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedDataExportAllocationStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_capped_data_export_allocation("code-949", "name-604");
        item.status = CappedDataExportAllocationStatus::Archived;
        assert!(
            item.move_to(CappedDataExportAllocationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
