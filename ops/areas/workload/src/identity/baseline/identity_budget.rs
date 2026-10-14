use super::*;

use acme_kit::text;

/// Where a identity budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl IdentityBudgetStatus {
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

impl std::fmt::Display for IdentityBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity budget as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBudget {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub cipher: String,
    pub owner_id: String,
    pub floor_value: f64,
    pub weight: f64,
    pub source: String,
    pub status: IdentityBudgetStatus,
}

impl IdentityBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityBudgetStatus) -> bool {
        match self.status {
            IdentityBudgetStatus::Draft => {
                match next {
                    IdentityBudgetStatus::Pending | IdentityBudgetStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityBudgetStatus::Pending => {
                return next == IdentityBudgetStatus::Closed;
            }
            IdentityBudgetStatus::Active => {
                return next == IdentityBudgetStatus::Suspended;
            }
            IdentityBudgetStatus::Suspended => {
                match next {
                    IdentityBudgetStatus::Closed => {
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

    pub fn move_to(&mut self, next: IdentityBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.floor_value - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityBudget {
        IdentityBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            cipher: self.cipher.clone(),
            owner_id: self.owner_id.clone(),
            floor_value: self.floor_value,
            weight: self.weight,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a identity budget in status draft.
pub fn new_identity_budget(code: &str, title: &str) -> IdentityBudget {
    IdentityBudget {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        cipher: "".to_string(),
        owner_id: "".to_string(),
        floor_value: 0.0,
        weight: 0.0,
        source: "".to_string(),
        status: IdentityBudgetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_identity_budget("code-542", "title-479");
        assert_eq!(item.status, IdentityBudgetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_identity_budget("code-542", "title-479");
        if item.move_to(IdentityBudgetStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityBudgetStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_identity_budget("code-542", "title-479");
        item.status = IdentityBudgetStatus::Closed;
        assert!(
            item.move_to(IdentityBudgetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
