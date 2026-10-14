use super::*;

use acme_kit::text;

/// Where a runbook preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookPreferenceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RunbookPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RunbookPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a runbook preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookPreference {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub quantity: i64,
    pub budget: f64,
    pub ratio: f64,
    pub updated_at: i64,
    pub status: RunbookPreferenceStatus,
}

impl RunbookPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookPreferenceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookPreferenceStatus) -> bool {
        match self.status {
            RunbookPreferenceStatus::Proposed => {
                match next {
                    RunbookPreferenceStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookPreferenceStatus::Approved => {
                return next == RunbookPreferenceStatus::Withdrawn;
            }
            RunbookPreferenceStatus::Rejected => {
                return next == RunbookPreferenceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.updated_at as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookPreference {
        RunbookPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            quantity: self.quantity,
            budget: self.budget,
            ratio: self.ratio,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a runbook preference in status proposed.
pub fn new_runbook_preference(code: &str, label: &str) -> RunbookPreference {
    RunbookPreference {
        code: code.to_string(),
        label: label.to_string(),
        notes: "".to_string(),
        quantity: 0,
        budget: 0.0,
        ratio: 0.0,
        updated_at: 0,
        status: RunbookPreferenceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_runbook_preference("code-204", "label-97");
        assert_eq!(item.status, RunbookPreferenceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_runbook_preference("code-204", "label-97");
        if item.move_to(RunbookPreferenceStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookPreferenceStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_runbook_preference("code-204", "label-97");
        item.status = RunbookPreferenceStatus::Withdrawn;
        assert!(
            item.move_to(RunbookPreferenceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
