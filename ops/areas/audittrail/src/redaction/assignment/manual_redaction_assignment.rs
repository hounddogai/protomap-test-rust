use super::*;

use acme_kit::text;

/// Where a manual redaction assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRedactionAssignmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualRedactionAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualRedactionAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual redaction assignment as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionAssignment {
    pub code: String,
    pub name: String,
    pub label: String,
    pub rate: f64,
    pub discount: f64,
    pub quota_units: i64,
    pub weight: f64,
    pub locale: String,
    pub status: ManualRedactionAssignmentStatus,
}

impl ManualRedactionAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRedactionAssignmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRedactionAssignmentStatus) -> bool {
        match self.status {
            ManualRedactionAssignmentStatus::Proposed => {
                match next {
                    ManualRedactionAssignmentStatus::Rejected | ManualRedactionAssignmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRedactionAssignmentStatus::Approved => {
                match next {
                    ManualRedactionAssignmentStatus::Rejected | ManualRedactionAssignmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRedactionAssignmentStatus::Rejected => {
                return next == ManualRedactionAssignmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualRedactionAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual redaction assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.weight - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRedactionAssignment {
        ManualRedactionAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            rate: self.rate,
            discount: self.discount,
            quota_units: self.quota_units,
            weight: self.weight,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a manual redaction assignment in status proposed.
pub fn new_manual_redaction_assignment(code: &str, name: &str) -> ManualRedactionAssignment {
    ManualRedactionAssignment {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        rate: 0.0,
        discount: 0.0,
        quota_units: 0,
        weight: 0.0,
        locale: "".to_string(),
        status: ManualRedactionAssignmentStatus::Proposed,
    }
}
