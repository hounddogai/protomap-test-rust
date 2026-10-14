use super::*;

use acme_kit::text;

/// Where a capped delegation draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedDelegationDraftStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedDelegationDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for CappedDelegationDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped delegation draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDelegationDraft {
    pub code: String,
    pub label: String,
    pub category: String,
    pub updated_at: i64,
    pub duration_minutes: i64,
    pub title: String,
    pub country: String,
    pub created_at: i64,
    pub status: CappedDelegationDraftStatus,
}

impl CappedDelegationDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedDelegationDraftStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedDelegationDraftStatus) -> bool {
        match self.status {
            CappedDelegationDraftStatus::Draft => {
                match next {
                    CappedDelegationDraftStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedDelegationDraftStatus::Pending => {
                return next == CappedDelegationDraftStatus::Active || next == CappedDelegationDraftStatus::Suspended;
            }
            CappedDelegationDraftStatus::Active => {
                return next == CappedDelegationDraftStatus::Suspended;
            }
            CappedDelegationDraftStatus::Suspended => {
                return next == CappedDelegationDraftStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedDelegationDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped delegation draft {} cannot move from {} to {}",
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

    pub fn effective_load(&self) -> f64 {
        ((self.created_at as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedDelegationDraft {
        CappedDelegationDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            updated_at: self.updated_at,
            duration_minutes: self.duration_minutes,
            title: self.title.clone(),
            country: self.country.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a capped delegation draft in status draft.
pub fn new_capped_delegation_draft(code: &str, label: &str) -> CappedDelegationDraft {
    CappedDelegationDraft {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        updated_at: 0,
        duration_minutes: 0,
        title: "".to_string(),
        country: "".to_string(),
        created_at: 0,
        status: CappedDelegationDraftStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_capped_delegation_draft("code-421", "label-620");
        assert_eq!(item.status, CappedDelegationDraftStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_capped_delegation_draft("code-421", "label-620");
        if item.move_to(CappedDelegationDraftStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedDelegationDraftStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_capped_delegation_draft("code-421", "label-620");
        item.status = CappedDelegationDraftStatus::Closed;
        assert!(
            item.move_to(CappedDelegationDraftStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
