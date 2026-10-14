use super::*;

use acme_kit::text;

/// Where a budget burn notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnNoticeStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BudgetBurnNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BudgetBurnNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a budget burn notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnNotice {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub namespace: String,
    pub pinned: bool,
    pub priority: i64,
    pub sequence: i64,
    pub description: String,
    pub created_at: i64,
    pub quota_units: i64,
    pub capacity: i64,
    pub status: BudgetBurnNoticeStatus,
}

impl BudgetBurnNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnNoticeStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnNoticeStatus) -> bool {
        match self.status {
            BudgetBurnNoticeStatus::Requested => {
                match next {
                    BudgetBurnNoticeStatus::Granted | BudgetBurnNoticeStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnNoticeStatus::Granted => {
                return next == BudgetBurnNoticeStatus::Revoked || next == BudgetBurnNoticeStatus::Expired;
            }
            BudgetBurnNoticeStatus::Revoked => {
                match next {
                    BudgetBurnNoticeStatus::Expired => {
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

    pub fn move_to(&mut self, next: BudgetBurnNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn notice {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.quota_units as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnNotice {
        BudgetBurnNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            namespace: self.namespace.clone(),
            pinned: self.pinned,
            priority: self.priority,
            sequence: self.sequence,
            description: self.description.clone(),
            created_at: self.created_at,
            quota_units: self.quota_units,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a budget burn notice in status requested.
pub fn new_budget_burn_notice(code: &str, label: &str) -> BudgetBurnNotice {
    BudgetBurnNotice {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        namespace: "".to_string(),
        pinned: false,
        priority: 0,
        sequence: 0,
        description: "".to_string(),
        created_at: 0,
        quota_units: 0,
        capacity: 0,
        status: BudgetBurnNoticeStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_budget_burn_notice("code-219", "label-810");
        assert_eq!(item.status, BudgetBurnNoticeStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_budget_burn_notice("code-219", "label-810");
        if item.move_to(BudgetBurnNoticeStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnNoticeStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_budget_burn_notice("code-219", "label-810");
        item.status = BudgetBurnNoticeStatus::Expired;
        assert!(
            item.move_to(BudgetBurnNoticeStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
