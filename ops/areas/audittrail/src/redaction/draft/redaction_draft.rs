use super::*;

use acme_kit::text;

/// Where a redaction draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RedactionDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RedactionDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a redaction draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionDraft {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub host: String,
    pub batch_size: i64,
    pub phone: String,
    pub title: String,
    pub unit: String,
    pub grace_days: i64,
    pub owner_id: String,
    pub quota_units: i64,
    pub status: RedactionDraftStatus,
}

impl RedactionDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionDraftStatus) -> bool {
        match self.status {
            RedactionDraftStatus::Created => {
                match next {
                    RedactionDraftStatus::Validated | RedactionDraftStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionDraftStatus::Validated => {
                return next == RedactionDraftStatus::Published || next == RedactionDraftStatus::Retired;
            }
            RedactionDraftStatus::Published => {
                match next {
                    RedactionDraftStatus::Retired => {
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

    pub fn move_to(&mut self, next: RedactionDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.grace_days as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionDraft {
        RedactionDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            host: self.host.clone(),
            batch_size: self.batch_size,
            phone: self.phone.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            grace_days: self.grace_days,
            owner_id: self.owner_id.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a redaction draft in status created.
pub fn new_redaction_draft(code: &str, label: &str) -> RedactionDraft {
    RedactionDraft {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        host: "".to_string(),
        batch_size: 0,
        phone: "".to_string(),
        title: "".to_string(),
        unit: "".to_string(),
        grace_days: 0,
        owner_id: "".to_string(),
        quota_units: 0,
        status: RedactionDraftStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_redaction_draft("code-484", "label-262");
        assert_eq!(item.status, RedactionDraftStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_redaction_draft("code-484", "label-262");
        if item.move_to(RedactionDraftStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionDraftStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_redaction_draft("code-484", "label-262");
        item.status = RedactionDraftStatus::Retired;
        assert!(
            item.move_to(RedactionDraftStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
