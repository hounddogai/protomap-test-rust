use super::*;

use acme_kit::text;

/// Where a fallback freeze draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackFreezeDraftStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackFreezeDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for FallbackFreezeDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback freeze draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFreezeDraft {
    pub code: String,
    pub name: String,
    pub capacity: i64,
    pub owner_id: String,
    pub currency: String,
    pub revision: i64,
    pub expires_at: i64,
    pub notes: String,
    pub status: FallbackFreezeDraftStatus,
}

impl FallbackFreezeDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackFreezeDraftStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackFreezeDraftStatus) -> bool {
        match self.status {
            FallbackFreezeDraftStatus::Planned => {
                match next {
                    FallbackFreezeDraftStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFreezeDraftStatus::Scheduled => {
                match next {
                    FallbackFreezeDraftStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFreezeDraftStatus::InProgress => {
                match next {
                    FallbackFreezeDraftStatus::Completed | FallbackFreezeDraftStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFreezeDraftStatus::Completed => {
                return next == FallbackFreezeDraftStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackFreezeDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback freeze draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.revision as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackFreezeDraft {
        FallbackFreezeDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            owner_id: self.owner_id.clone(),
            currency: self.currency.clone(),
            revision: self.revision,
            expires_at: self.expires_at,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback freeze draft in status planned.
pub fn new_fallback_freeze_draft(code: &str, name: &str) -> FallbackFreezeDraft {
    FallbackFreezeDraft {
        code: code.to_string(),
        name: name.to_string(),
        capacity: 0,
        owner_id: "".to_string(),
        currency: "".to_string(),
        revision: 0,
        expires_at: 0,
        notes: "".to_string(),
        status: FallbackFreezeDraftStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_freeze_draft("code-391", "name-668");
        assert_eq!(item.status, FallbackFreezeDraftStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_fallback_freeze_draft("code-391", "name-668");
        if item.move_to(FallbackFreezeDraftStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackFreezeDraftStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_freeze_draft("code-391", "name-668");
        item.status = FallbackFreezeDraftStatus::Cancelled;
        assert!(
            item.move_to(FallbackFreezeDraftStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
