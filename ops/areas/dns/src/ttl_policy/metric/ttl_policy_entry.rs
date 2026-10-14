use super::*;

use acme_kit::text;

/// Where a ttl policy entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TtlPolicyEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TtlPolicyEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy entry as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyEntry {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub window_days: i64,
    pub owner_id: String,
    pub category: String,
    pub sequence: i64,
    pub status: TtlPolicyEntryStatus,
}

impl TtlPolicyEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyEntryStatus) -> bool {
        match self.status {
            TtlPolicyEntryStatus::Proposed => {
                return next == TtlPolicyEntryStatus::Rejected || next == TtlPolicyEntryStatus::Withdrawn;
            }
            TtlPolicyEntryStatus::Approved => {
                match next {
                    TtlPolicyEntryStatus::Rejected | TtlPolicyEntryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyEntryStatus::Rejected => {
                match next {
                    TtlPolicyEntryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TtlPolicyEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "ttl policy entry", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.sequence as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyEntry {
        TtlPolicyEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            window_days: self.window_days,
            owner_id: self.owner_id.clone(),
            category: self.category.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a ttl policy entry in status proposed.
pub fn new_ttl_policy_entry(code: &str, name: &str) -> TtlPolicyEntry {
    TtlPolicyEntry {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        window_days: 0,
        owner_id: "".to_string(),
        category: "".to_string(),
        sequence: 0,
        status: TtlPolicyEntryStatus::Proposed,
    }
}
