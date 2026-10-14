use super::*;

use acme_kit::text;

/// Where a seasonal escalation agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalEscalationAgreementStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalEscalationAgreementStatus {
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

impl std::fmt::Display for SeasonalEscalationAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal escalation agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEscalationAgreement {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub country: String,
    pub timezone: String,
    pub limit: i64,
    pub budget: f64,
    pub status: SeasonalEscalationAgreementStatus,
}

impl SeasonalEscalationAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalEscalationAgreementStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalEscalationAgreementStatus) -> bool {
        match self.status {
            SeasonalEscalationAgreementStatus::Idle => {
                return next == SeasonalEscalationAgreementStatus::Draining
                    || next == SeasonalEscalationAgreementStatus::Stopped;
            }
            SeasonalEscalationAgreementStatus::Warming => {
                match next {
                    SeasonalEscalationAgreementStatus::Draining | SeasonalEscalationAgreementStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalEscalationAgreementStatus::Ready => {
                return next == SeasonalEscalationAgreementStatus::Draining
                    || next == SeasonalEscalationAgreementStatus::Stopped;
            }
            SeasonalEscalationAgreementStatus::Draining => {
                return next == SeasonalEscalationAgreementStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalEscalationAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal escalation agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.budget + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalEscalationAgreement {
        SeasonalEscalationAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            country: self.country.clone(),
            timezone: self.timezone.clone(),
            limit: self.limit,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a seasonal escalation agreement in status idle.
pub fn new_seasonal_escalation_agreement(code: &str, name: &str) -> SeasonalEscalationAgreement {
    SeasonalEscalationAgreement {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        country: "".to_string(),
        timezone: "".to_string(),
        limit: 0,
        budget: 0.0,
        status: SeasonalEscalationAgreementStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_seasonal_escalation_agreement("code-711", "name-873");
        assert_eq!(item.status, SeasonalEscalationAgreementStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_seasonal_escalation_agreement("code-711", "name-873");
        if item.move_to(SeasonalEscalationAgreementStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalEscalationAgreementStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_seasonal_escalation_agreement("code-711", "name-873");
        item.status = SeasonalEscalationAgreementStatus::Stopped;
        assert!(
            item.move_to(SeasonalEscalationAgreementStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
