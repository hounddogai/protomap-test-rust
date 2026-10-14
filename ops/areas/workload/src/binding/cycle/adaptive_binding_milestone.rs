use super::*;

use acme_kit::text;

/// Where a adaptive binding milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveBindingMilestoneStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveBindingMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveBindingMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive binding milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBindingMilestone {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub ratio: f64,
    pub ceiling: f64,
    pub audience: String,
    pub currency: String,
    pub channel: String,
    pub issuer: String,
    pub attempts: i64,
    pub status: AdaptiveBindingMilestoneStatus,
}

impl AdaptiveBindingMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveBindingMilestoneStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveBindingMilestoneStatus) -> bool {
        match self.status {
            AdaptiveBindingMilestoneStatus::Proposed => {
                match next {
                    AdaptiveBindingMilestoneStatus::Rejected | AdaptiveBindingMilestoneStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveBindingMilestoneStatus::Approved => {
                match next {
                    AdaptiveBindingMilestoneStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveBindingMilestoneStatus::Rejected => {
                match next {
                    AdaptiveBindingMilestoneStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AdaptiveBindingMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive binding milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.attempts as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveBindingMilestone {
        AdaptiveBindingMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            ratio: self.ratio,
            ceiling: self.ceiling,
            audience: self.audience.clone(),
            currency: self.currency.clone(),
            channel: self.channel.clone(),
            issuer: self.issuer.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a adaptive binding milestone in status proposed.
pub fn new_adaptive_binding_milestone(code: &str, title: &str) -> AdaptiveBindingMilestone {
    AdaptiveBindingMilestone {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        ratio: 0.0,
        ceiling: 0.0,
        audience: "".to_string(),
        currency: "".to_string(),
        channel: "".to_string(),
        issuer: "".to_string(),
        attempts: 0,
        status: AdaptiveBindingMilestoneStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_adaptive_binding_milestone("code-828", "title-709");
        assert_eq!(item.status, AdaptiveBindingMilestoneStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_adaptive_binding_milestone("code-828", "title-709");
        if item.move_to(AdaptiveBindingMilestoneStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveBindingMilestoneStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_adaptive_binding_milestone("code-828", "title-709");
        item.status = AdaptiveBindingMilestoneStatus::Withdrawn;
        assert!(
            item.move_to(AdaptiveBindingMilestoneStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
