use super::*;

use acme_kit::text;

/// Where a lenient data export milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientDataExportMilestoneStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LenientDataExportMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LenientDataExportMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient data export milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDataExportMilestone {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub external_ref: String,
    pub pinned: bool,
    pub status: LenientDataExportMilestoneStatus,
}

impl LenientDataExportMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientDataExportMilestoneStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientDataExportMilestoneStatus) -> bool {
        match self.status {
            LenientDataExportMilestoneStatus::Proposed => {
                return next == LenientDataExportMilestoneStatus::Approved
                    || next == LenientDataExportMilestoneStatus::Rejected;
            }
            LenientDataExportMilestoneStatus::Approved => {
                return next == LenientDataExportMilestoneStatus::Rejected
                    || next == LenientDataExportMilestoneStatus::Withdrawn;
            }
            LenientDataExportMilestoneStatus::Rejected => {
                return next == LenientDataExportMilestoneStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientDataExportMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient data export milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient data export milestone", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> LenientDataExportMilestone {
        LenientDataExportMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            external_ref: self.external_ref.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient data export milestone in status proposed.
pub fn new_lenient_data_export_milestone(code: &str, title: &str) -> LenientDataExportMilestone {
    LenientDataExportMilestone {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        external_ref: "".to_string(),
        pinned: false,
        status: LenientDataExportMilestoneStatus::Proposed,
    }
}
