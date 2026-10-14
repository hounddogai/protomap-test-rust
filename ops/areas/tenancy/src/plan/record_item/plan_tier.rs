use super::*;

use acme_kit::text;

/// Where a plan tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanTierStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PlanTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for PlanTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a plan tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanTier {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub unit: String,
    pub region: String,
    pub grace_days: i64,
    pub currency: String,
    pub status: PlanTierStatus,
}

impl PlanTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanTierStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanTierStatus) -> bool {
        match self.status {
            PlanTierStatus::Open => {
                match next {
                    PlanTierStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanTierStatus::InReview => {
                return next == PlanTierStatus::Resolved || next == PlanTierStatus::Archived;
            }
            PlanTierStatus::Resolved => {
                match next {
                    PlanTierStatus::Reopened | PlanTierStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanTierStatus::Reopened => {
                match next {
                    PlanTierStatus::Archived => {
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

    pub fn move_to(&mut self, next: PlanTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> PlanTier {
        PlanTier {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            unit: self.unit.clone(),
            region: self.region.clone(),
            grace_days: self.grace_days,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a plan tier in status open.
pub fn new_plan_tier(code: &str, label: &str) -> PlanTier {
    PlanTier {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        unit: "".to_string(),
        region: "".to_string(),
        grace_days: 0,
        currency: "".to_string(),
        status: PlanTierStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_plan_tier("code-139", "label-466");
        assert_eq!(item.status, PlanTierStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_plan_tier("code-139", "label-466");
        if item.move_to(PlanTierStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanTierStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_plan_tier("code-139", "label-466");
        item.status = PlanTierStatus::Archived;
        assert!(item.move_to(PlanTierStatus::Open).is_err(), "expected move to fail");
    }
}
