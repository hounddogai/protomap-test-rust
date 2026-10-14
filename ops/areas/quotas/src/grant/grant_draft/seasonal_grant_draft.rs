use super::*;

use acme_kit::text;

/// Where a seasonal grant draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalGrantDraftStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalGrantDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalGrantDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal grant draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantDraft {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub owner_id: String,
    pub created_at: i64,
    pub status: SeasonalGrantDraftStatus,
}

impl SeasonalGrantDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalGrantDraftStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalGrantDraftStatus) -> bool {
        match self.status {
            SeasonalGrantDraftStatus::Proposed => {
                match next {
                    SeasonalGrantDraftStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalGrantDraftStatus::Approved => {
                return next == SeasonalGrantDraftStatus::Rejected || next == SeasonalGrantDraftStatus::Withdrawn;
            }
            SeasonalGrantDraftStatus::Rejected => {
                match next {
                    SeasonalGrantDraftStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: SeasonalGrantDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal grant draft {} cannot move from {} to {}",
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
        format!("{} {}", "seasonal grant draft", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> SeasonalGrantDraft {
        SeasonalGrantDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            owner_id: self.owner_id.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a seasonal grant draft in status proposed.
pub fn new_seasonal_grant_draft(code: &str, title: &str) -> SeasonalGrantDraft {
    SeasonalGrantDraft {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        owner_id: "".to_string(),
        created_at: 0,
        status: SeasonalGrantDraftStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_seasonal_grant_draft("code-948", "title-839");
        assert_eq!(item.status, SeasonalGrantDraftStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_seasonal_grant_draft("code-948", "title-839");
        if item.move_to(SeasonalGrantDraftStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalGrantDraftStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_seasonal_grant_draft("code-948", "title-839");
        item.status = SeasonalGrantDraftStatus::Withdrawn;
        assert!(
            item.move_to(SeasonalGrantDraftStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
