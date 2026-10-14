use super::*;

use acme_kit::text;

/// Where a primary limit plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryLimitPlanStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryLimitPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryLimitPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary limit plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitPlan {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub threshold: f64,
    pub ratio: f64,
    pub ttl_seconds: i64,
    pub region: String,
    pub maximum: f64,
    pub status: PrimaryLimitPlanStatus,
}

impl PrimaryLimitPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryLimitPlanStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryLimitPlanStatus) -> bool {
        match self.status {
            PrimaryLimitPlanStatus::Requested => {
                return next == PrimaryLimitPlanStatus::Revoked || next == PrimaryLimitPlanStatus::Expired;
            }
            PrimaryLimitPlanStatus::Granted => {
                match next {
                    PrimaryLimitPlanStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryLimitPlanStatus::Revoked => {
                return next == PrimaryLimitPlanStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryLimitPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary limit plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary limit plan", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.threshold - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryLimitPlan {
        PrimaryLimitPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            threshold: self.threshold,
            ratio: self.ratio,
            ttl_seconds: self.ttl_seconds,
            region: self.region.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a primary limit plan in status requested.
pub fn new_primary_limit_plan(code: &str, label: &str) -> PrimaryLimitPlan {
    PrimaryLimitPlan {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        threshold: 0.0,
        ratio: 0.0,
        ttl_seconds: 0,
        region: "".to_string(),
        maximum: 0.0,
        status: PrimaryLimitPlanStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_limit_plan("code-469", "label-72");
        assert_eq!(item.status, PrimaryLimitPlanStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_primary_limit_plan("code-469", "label-72");
        if item.move_to(PrimaryLimitPlanStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryLimitPlanStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_limit_plan("code-469", "label-72");
        item.status = PrimaryLimitPlanStatus::Expired;
        assert!(
            item.move_to(PrimaryLimitPlanStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
