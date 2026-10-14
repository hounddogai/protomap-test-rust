use super::*;

use acme_kit::text;

/// Where a manual quote plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualQuotePlanStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualQuotePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualQuotePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual quote plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuotePlan {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub floor_value: f64,
    pub visible: bool,
    pub unit: String,
    pub scope: String,
    pub threshold: f64,
    pub status: ManualQuotePlanStatus,
}

impl ManualQuotePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualQuotePlanStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualQuotePlanStatus) -> bool {
        match self.status {
            ManualQuotePlanStatus::Requested => {
                return next == ManualQuotePlanStatus::Granted;
            }
            ManualQuotePlanStatus::Granted => {
                match next {
                    ManualQuotePlanStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualQuotePlanStatus::Revoked => {
                return next == ManualQuotePlanStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualQuotePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual quote plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.floor_value + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualQuotePlan {
        ManualQuotePlan {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            floor_value: self.floor_value,
            visible: self.visible,
            unit: self.unit.clone(),
            scope: self.scope.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a manual quote plan in status requested.
pub fn new_manual_quote_plan(code: &str, title: &str) -> ManualQuotePlan {
    ManualQuotePlan {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        floor_value: 0.0,
        visible: false,
        unit: "".to_string(),
        scope: "".to_string(),
        threshold: 0.0,
        status: ManualQuotePlanStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_quote_plan("code-745", "title-460");
        assert_eq!(item.status, ManualQuotePlanStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_manual_quote_plan("code-745", "title-460");
        if item.move_to(ManualQuotePlanStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualQuotePlanStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_quote_plan("code-745", "title-460");
        item.status = ManualQuotePlanStatus::Expired;
        assert!(
            item.move_to(ManualQuotePlanStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
