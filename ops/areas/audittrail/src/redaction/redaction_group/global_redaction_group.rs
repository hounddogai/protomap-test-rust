use super::*;

use acme_kit::text;

/// Where a global redaction group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalRedactionGroupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl GlobalRedactionGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for GlobalRedactionGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global redaction group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionGroup {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub reviewer: String,
    pub name: String,
    pub summary: String,
    pub quantity: i64,
    pub phone: String,
    pub status: GlobalRedactionGroupStatus,
}

impl GlobalRedactionGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalRedactionGroupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalRedactionGroupStatus) -> bool {
        match self.status {
            GlobalRedactionGroupStatus::Proposed => {
                match next {
                    GlobalRedactionGroupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalRedactionGroupStatus::Approved => {
                return next == GlobalRedactionGroupStatus::Withdrawn;
            }
            GlobalRedactionGroupStatus::Rejected => {
                return next == GlobalRedactionGroupStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalRedactionGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global redaction group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "global redaction group", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.retries as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalRedactionGroup {
        GlobalRedactionGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            reviewer: self.reviewer.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            quantity: self.quantity,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global redaction group in status proposed.
pub fn new_global_redaction_group(code: &str, label: &str) -> GlobalRedactionGroup {
    GlobalRedactionGroup {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        reviewer: "".to_string(),
        name: "".to_string(),
        summary: "".to_string(),
        quantity: 0,
        phone: "".to_string(),
        status: GlobalRedactionGroupStatus::Proposed,
    }
}
