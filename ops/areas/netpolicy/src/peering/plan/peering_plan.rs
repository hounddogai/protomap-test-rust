use super::*;

use acme_kit::text;

/// Where a peering plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringPlanStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PeeringPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PeeringPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a peering plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPlan {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub audience: String,
    pub active: bool,
    pub currency: String,
    pub score: f64,
    pub status: PeeringPlanStatus,
}

impl PeeringPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringPlanStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringPlanStatus) -> bool {
        match self.status {
            PeeringPlanStatus::Requested => {
                return next == PeeringPlanStatus::Revoked || next == PeeringPlanStatus::Expired;
            }
            PeeringPlanStatus::Granted => {
                match next {
                    PeeringPlanStatus::Revoked | PeeringPlanStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringPlanStatus::Revoked => {
                match next {
                    PeeringPlanStatus::Expired => {
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

    pub fn move_to(&mut self, next: PeeringPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> PeeringPlan {
        PeeringPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            audience: self.audience.clone(),
            active: self.active,
            currency: self.currency.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a peering plan in status requested.
pub fn new_peering_plan(code: &str, title: &str) -> PeeringPlan {
    PeeringPlan {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        audience: "".to_string(),
        active: false,
        currency: "".to_string(),
        score: 0.0,
        status: PeeringPlanStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_peering_plan("code-355", "title-514");
        assert_eq!(item.status, PeeringPlanStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_peering_plan("code-355", "title-514");
        if item.move_to(PeeringPlanStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringPlanStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_peering_plan("code-355", "title-514");
        item.status = PeeringPlanStatus::Expired;
        assert!(
            item.move_to(PeeringPlanStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
