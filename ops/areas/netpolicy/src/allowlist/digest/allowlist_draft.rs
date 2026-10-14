use super::*;

use acme_kit::text;

/// Where a allowlist draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistDraftStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AllowlistDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AllowlistDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a allowlist draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistDraft {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub amount: f64,
    pub active: bool,
    pub category: String,
    pub floor_value: f64,
    pub window_days: i64,
    pub status: AllowlistDraftStatus,
}

impl AllowlistDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistDraftStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistDraftStatus) -> bool {
        match self.status {
            AllowlistDraftStatus::Requested => {
                return next == AllowlistDraftStatus::Granted || next == AllowlistDraftStatus::Expired;
            }
            AllowlistDraftStatus::Granted => {
                return next == AllowlistDraftStatus::Revoked || next == AllowlistDraftStatus::Expired;
            }
            AllowlistDraftStatus::Revoked => {
                return next == AllowlistDraftStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        (self.amount * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistDraft {
        AllowlistDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            amount: self.amount,
            active: self.active,
            category: self.category.clone(),
            floor_value: self.floor_value,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a allowlist draft in status requested.
pub fn new_allowlist_draft(code: &str, name: &str) -> AllowlistDraft {
    AllowlistDraft {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        amount: 0.0,
        active: false,
        category: "".to_string(),
        floor_value: 0.0,
        window_days: 0,
        status: AllowlistDraftStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_allowlist_draft("code-644", "name-406");
        assert_eq!(item.status, AllowlistDraftStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_allowlist_draft("code-644", "name-406");
        if item.move_to(AllowlistDraftStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistDraftStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_allowlist_draft("code-644", "name-406");
        item.status = AllowlistDraftStatus::Expired;
        assert!(
            item.move_to(AllowlistDraftStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
