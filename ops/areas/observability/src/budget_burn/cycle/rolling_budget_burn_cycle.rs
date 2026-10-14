use super::*;

use acme_kit::text;

/// Where a rolling budget burn cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingBudgetBurnCycleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingBudgetBurnCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingBudgetBurnCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling budget burn cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnCycle {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub limit: i64,
    pub ttl_seconds: i64,
    pub cipher: String,
    pub ceiling: f64,
    pub locale: String,
    pub version: i64,
    pub retries: i64,
    pub contact_email: String,
    pub status: RollingBudgetBurnCycleStatus,
}

impl RollingBudgetBurnCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingBudgetBurnCycleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingBudgetBurnCycleStatus) -> bool {
        match self.status {
            RollingBudgetBurnCycleStatus::Requested => {
                match next {
                    RollingBudgetBurnCycleStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingBudgetBurnCycleStatus::Granted => {
                match next {
                    RollingBudgetBurnCycleStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingBudgetBurnCycleStatus::Revoked => {
                return next == RollingBudgetBurnCycleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingBudgetBurnCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling budget burn cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.retries as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingBudgetBurnCycle {
        RollingBudgetBurnCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            limit: self.limit,
            ttl_seconds: self.ttl_seconds,
            cipher: self.cipher.clone(),
            ceiling: self.ceiling,
            locale: self.locale.clone(),
            version: self.version,
            retries: self.retries,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a rolling budget burn cycle in status requested.
pub fn new_rolling_budget_burn_cycle(code: &str, name: &str) -> RollingBudgetBurnCycle {
    RollingBudgetBurnCycle {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        limit: 0,
        ttl_seconds: 0,
        cipher: "".to_string(),
        ceiling: 0.0,
        locale: "".to_string(),
        version: 0,
        retries: 0,
        contact_email: "".to_string(),
        status: RollingBudgetBurnCycleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rolling_budget_burn_cycle("code-50", "name-167");
        assert_eq!(item.status, RollingBudgetBurnCycleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_rolling_budget_burn_cycle("code-50", "name-167");
        if item.move_to(RollingBudgetBurnCycleStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingBudgetBurnCycleStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rolling_budget_burn_cycle("code-50", "name-167");
        item.status = RollingBudgetBurnCycleStatus::Expired;
        assert!(
            item.move_to(RollingBudgetBurnCycleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
