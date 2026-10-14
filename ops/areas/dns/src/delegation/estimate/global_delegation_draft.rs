use super::*;

use acme_kit::text;

/// Where a global delegation draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalDelegationDraftStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalDelegationDraftStatus {
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

impl std::fmt::Display for GlobalDelegationDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global delegation draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDelegationDraft {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub notes: String,
    pub limit: i64,
    pub key_id: String,
    pub status: GlobalDelegationDraftStatus,
}

impl GlobalDelegationDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalDelegationDraftStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalDelegationDraftStatus) -> bool {
        match self.status {
            GlobalDelegationDraftStatus::Draft => {
                match next {
                    GlobalDelegationDraftStatus::Active | GlobalDelegationDraftStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalDelegationDraftStatus::Pending => {
                return next == GlobalDelegationDraftStatus::Active;
            }
            GlobalDelegationDraftStatus::Active => {
                return next == GlobalDelegationDraftStatus::Suspended || next == GlobalDelegationDraftStatus::Closed;
            }
            GlobalDelegationDraftStatus::Suspended => {
                return next == GlobalDelegationDraftStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalDelegationDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global delegation draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.discount - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalDelegationDraft {
        GlobalDelegationDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            notes: self.notes.clone(),
            limit: self.limit,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global delegation draft in status draft.
pub fn new_global_delegation_draft(code: &str, title: &str) -> GlobalDelegationDraft {
    GlobalDelegationDraft {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        notes: "".to_string(),
        limit: 0,
        key_id: "".to_string(),
        status: GlobalDelegationDraftStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_delegation_draft("code-497", "title-293");
        assert_eq!(item.status, GlobalDelegationDraftStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_global_delegation_draft("code-497", "title-293");
        if item.move_to(GlobalDelegationDraftStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalDelegationDraftStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_delegation_draft("code-497", "title-293");
        item.status = GlobalDelegationDraftStatus::Closed;
        assert!(
            item.move_to(GlobalDelegationDraftStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
