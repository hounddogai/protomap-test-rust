use super::*;

use acme_kit::text;

/// Where a tiered budget burn profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredBudgetBurnProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TieredBudgetBurnProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TieredBudgetBurnProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered budget burn profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnProfile {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub host: String,
    pub principal: String,
    pub namespace: String,
    pub budget: f64,
    pub rotation_days: i64,
    pub status: TieredBudgetBurnProfileStatus,
}

impl TieredBudgetBurnProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredBudgetBurnProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredBudgetBurnProfileStatus) -> bool {
        match self.status {
            TieredBudgetBurnProfileStatus::Requested => {
                return next == TieredBudgetBurnProfileStatus::Revoked;
            }
            TieredBudgetBurnProfileStatus::Granted => {
                return next == TieredBudgetBurnProfileStatus::Revoked
                    || next == TieredBudgetBurnProfileStatus::Expired;
            }
            TieredBudgetBurnProfileStatus::Revoked => {
                return next == TieredBudgetBurnProfileStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredBudgetBurnProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered budget burn profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.rotation_days as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredBudgetBurnProfile {
        TieredBudgetBurnProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            host: self.host.clone(),
            principal: self.principal.clone(),
            namespace: self.namespace.clone(),
            budget: self.budget,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a tiered budget burn profile in status requested.
pub fn new_tiered_budget_burn_profile(code: &str, title: &str) -> TieredBudgetBurnProfile {
    TieredBudgetBurnProfile {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        host: "".to_string(),
        principal: "".to_string(),
        namespace: "".to_string(),
        budget: 0.0,
        rotation_days: 0,
        status: TieredBudgetBurnProfileStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_tiered_budget_burn_profile("code-536", "title-387");
        assert_eq!(item.status, TieredBudgetBurnProfileStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_tiered_budget_burn_profile("code-536", "title-387");
        if item.move_to(TieredBudgetBurnProfileStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredBudgetBurnProfileStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_tiered_budget_burn_profile("code-536", "title-387");
        item.status = TieredBudgetBurnProfileStatus::Expired;
        assert!(
            item.move_to(TieredBudgetBurnProfileStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
