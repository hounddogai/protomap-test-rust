use super::*;

use acme_kit::text;

/// Where a binding draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingDraftStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BindingDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BindingDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One binding draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingDraft {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub cost: f64,
    pub starts_at: i64,
    pub category: String,
    pub weight: f64,
    pub notes: String,
    pub burst: i64,
    pub amount: f64,
    pub quota_units: i64,
    pub status: BindingDraftStatus,
}

impl BindingDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingDraftStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingDraftStatus) -> bool {
        match self.status {
            BindingDraftStatus::Proposed => {
                match next {
                    BindingDraftStatus::Rejected | BindingDraftStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingDraftStatus::Approved => {
                match next {
                    BindingDraftStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingDraftStatus::Rejected => {
                return next == BindingDraftStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "binding draft", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.starts_at as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingDraft {
        BindingDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            cost: self.cost,
            starts_at: self.starts_at,
            category: self.category.clone(),
            weight: self.weight,
            notes: self.notes.clone(),
            burst: self.burst,
            amount: self.amount,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a binding draft in status proposed.
pub fn new_binding_draft(code: &str, name: &str) -> BindingDraft {
    BindingDraft {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        cost: 0.0,
        starts_at: 0,
        category: "".to_string(),
        weight: 0.0,
        notes: "".to_string(),
        burst: 0,
        amount: 0.0,
        quota_units: 0,
        status: BindingDraftStatus::Proposed,
    }
}
