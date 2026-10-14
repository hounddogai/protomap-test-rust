use super::*;

use acme_kit::text;

/// Where a scheduled delegation score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledDelegationScoreStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledDelegationScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ScheduledDelegationScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled delegation score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDelegationScore {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub channel: String,
    pub rotation_days: i64,
    pub ratio: f64,
    pub owner_id: String,
    pub description: String,
    pub timezone: String,
    pub threshold: f64,
    pub status: ScheduledDelegationScoreStatus,
}

impl ScheduledDelegationScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledDelegationScoreStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledDelegationScoreStatus) -> bool {
        match self.status {
            ScheduledDelegationScoreStatus::Idle => {
                return next == ScheduledDelegationScoreStatus::Ready
                    || next == ScheduledDelegationScoreStatus::Draining;
            }
            ScheduledDelegationScoreStatus::Warming => {
                match next {
                    ScheduledDelegationScoreStatus::Draining | ScheduledDelegationScoreStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledDelegationScoreStatus::Ready => {
                return next == ScheduledDelegationScoreStatus::Draining
                    || next == ScheduledDelegationScoreStatus::Stopped;
            }
            ScheduledDelegationScoreStatus::Draining => {
                match next {
                    ScheduledDelegationScoreStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ScheduledDelegationScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled delegation score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled delegation score", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        (self.threshold * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledDelegationScore {
        ScheduledDelegationScore {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            channel: self.channel.clone(),
            rotation_days: self.rotation_days,
            ratio: self.ratio,
            owner_id: self.owner_id.clone(),
            description: self.description.clone(),
            timezone: self.timezone.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a scheduled delegation score in status idle.
pub fn new_scheduled_delegation_score(code: &str, label: &str) -> ScheduledDelegationScore {
    ScheduledDelegationScore {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        channel: "".to_string(),
        rotation_days: 0,
        ratio: 0.0,
        owner_id: "".to_string(),
        description: "".to_string(),
        timezone: "".to_string(),
        threshold: 0.0,
        status: ScheduledDelegationScoreStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_delegation_score("code-928", "label-443");
        assert_eq!(item.status, ScheduledDelegationScoreStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_scheduled_delegation_score("code-928", "label-443");
        if item.move_to(ScheduledDelegationScoreStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledDelegationScoreStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_delegation_score("code-928", "label-443");
        item.status = ScheduledDelegationScoreStatus::Stopped;
        assert!(
            item.move_to(ScheduledDelegationScoreStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
