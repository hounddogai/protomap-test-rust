use super::*;

use acme_kit::text;

/// Where a budget burn baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnBaselineStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BudgetBurnBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BudgetBurnBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One budget burn baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBaseline {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub contact_email: String,
    pub phone: String,
    pub starts_at: i64,
    pub ttl_seconds: i64,
    pub channel: String,
    pub budget: f64,
    pub revision: i64,
    pub status: BudgetBurnBaselineStatus,
}

impl BudgetBurnBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnBaselineStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnBaselineStatus) -> bool {
        match self.status {
            BudgetBurnBaselineStatus::Requested => {
                return next == BudgetBurnBaselineStatus::Granted || next == BudgetBurnBaselineStatus::Revoked;
            }
            BudgetBurnBaselineStatus::Granted => {
                match next {
                    BudgetBurnBaselineStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnBaselineStatus::Revoked => {
                match next {
                    BudgetBurnBaselineStatus::Expired => {
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

    pub fn move_to(&mut self, next: BudgetBurnBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnBaseline {
        BudgetBurnBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            contact_email: self.contact_email.clone(),
            phone: self.phone.clone(),
            starts_at: self.starts_at,
            ttl_seconds: self.ttl_seconds,
            channel: self.channel.clone(),
            budget: self.budget,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a budget burn baseline in status requested.
pub fn new_budget_burn_baseline(code: &str, title: &str) -> BudgetBurnBaseline {
    BudgetBurnBaseline {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        contact_email: "".to_string(),
        phone: "".to_string(),
        starts_at: 0,
        ttl_seconds: 0,
        channel: "".to_string(),
        budget: 0.0,
        revision: 0,
        status: BudgetBurnBaselineStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_budget_burn_baseline("code-496", "title-962");
        assert_eq!(item.status, BudgetBurnBaselineStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_budget_burn_baseline("code-496", "title-962");
        if item.move_to(BudgetBurnBaselineStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnBaselineStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_budget_burn_baseline("code-496", "title-962");
        item.status = BudgetBurnBaselineStatus::Expired;
        assert!(
            item.move_to(BudgetBurnBaselineStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
