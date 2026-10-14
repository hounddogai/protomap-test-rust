use super::*;

use acme_kit::text;

/// Where a health check proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckProposalStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl HealthCheckProposalStatus {
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

impl std::fmt::Display for HealthCheckProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a health check proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckProposal {
    pub code: String,
    pub label: String,
    pub ratio: f64,
    pub batch_size: i64,
    pub cost: f64,
    pub host: String,
    pub archived: bool,
    pub unit: String,
    pub retries: i64,
    pub attempts: i64,
    pub phone: String,
    pub status: HealthCheckProposalStatus,
}

impl HealthCheckProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckProposalStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckProposalStatus) -> bool {
        match self.status {
            HealthCheckProposalStatus::Idle => {
                return next == HealthCheckProposalStatus::Ready || next == HealthCheckProposalStatus::Stopped;
            }
            HealthCheckProposalStatus::Warming => {
                return next == HealthCheckProposalStatus::Ready || next == HealthCheckProposalStatus::Stopped;
            }
            HealthCheckProposalStatus::Ready => {
                return next == HealthCheckProposalStatus::Stopped;
            }
            HealthCheckProposalStatus::Draining => {
                return next == HealthCheckProposalStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "health check proposal", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.retries as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckProposal {
        HealthCheckProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            ratio: self.ratio,
            batch_size: self.batch_size,
            cost: self.cost,
            host: self.host.clone(),
            archived: self.archived,
            unit: self.unit.clone(),
            retries: self.retries,
            attempts: self.attempts,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a health check proposal in status idle.
pub fn new_health_check_proposal(code: &str, label: &str) -> HealthCheckProposal {
    HealthCheckProposal {
        code: code.to_string(),
        label: label.to_string(),
        ratio: 0.0,
        batch_size: 0,
        cost: 0.0,
        host: "".to_string(),
        archived: false,
        unit: "".to_string(),
        retries: 0,
        attempts: 0,
        phone: "".to_string(),
        status: HealthCheckProposalStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_health_check_proposal("code-658", "label-754");
        assert_eq!(item.status, HealthCheckProposalStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_health_check_proposal("code-658", "label-754");
        if item.move_to(HealthCheckProposalStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckProposalStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_health_check_proposal("code-658", "label-754");
        item.status = HealthCheckProposalStatus::Stopped;
        assert!(
            item.move_to(HealthCheckProposalStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
