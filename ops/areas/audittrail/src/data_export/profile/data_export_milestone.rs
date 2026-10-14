use super::*;

use acme_kit::text;

/// Where a data export milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportMilestoneStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DataExportMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DataExportMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a data export milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMilestone {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub segment: String,
    pub cost: f64,
    pub ceiling: f64,
    pub status: DataExportMilestoneStatus,
}

impl DataExportMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportMilestoneStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportMilestoneStatus) -> bool {
        match self.status {
            DataExportMilestoneStatus::Proposed => {
                return next == DataExportMilestoneStatus::Rejected;
            }
            DataExportMilestoneStatus::Approved => {
                match next {
                    DataExportMilestoneStatus::Rejected | DataExportMilestoneStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportMilestoneStatus::Rejected => {
                return next == DataExportMilestoneStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.ceiling - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportMilestone {
        DataExportMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            segment: self.segment.clone(),
            cost: self.cost,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a data export milestone in status proposed.
pub fn new_data_export_milestone(code: &str, label: &str) -> DataExportMilestone {
    DataExportMilestone {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        segment: "".to_string(),
        cost: 0.0,
        ceiling: 0.0,
        status: DataExportMilestoneStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_data_export_milestone("code-956", "label-761");
        assert_eq!(item.status, DataExportMilestoneStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_data_export_milestone("code-956", "label-761");
        if item.move_to(DataExportMilestoneStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportMilestoneStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_data_export_milestone("code-956", "label-761");
        item.status = DataExportMilestoneStatus::Withdrawn;
        assert!(
            item.move_to(DataExportMilestoneStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
