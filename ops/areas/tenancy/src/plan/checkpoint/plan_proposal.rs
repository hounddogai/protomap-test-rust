use super::*;

use acme_kit::text;

/// Where a plan proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanProposalStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PlanProposalStatus {
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

impl std::fmt::Display for PlanProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan proposal as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanProposal {
    pub code: String,
    pub title: String,
    pub description: String,
    pub threshold: f64,
    pub window_days: i64,
    pub limit: i64,
    pub burst: i64,
    pub unit: String,
    pub locale: String,
    pub budget: f64,
    pub port: i64,
    pub status: PlanProposalStatus,
}

impl PlanProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanProposalStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanProposalStatus) -> bool {
        match self.status {
            PlanProposalStatus::Idle => {
                return next == PlanProposalStatus::Warming;
            }
            PlanProposalStatus::Warming => {
                match next {
                    PlanProposalStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanProposalStatus::Ready => {
                match next {
                    PlanProposalStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanProposalStatus::Draining => {
                match next {
                    PlanProposalStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PlanProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "plan proposal", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.limit as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanProposal {
        PlanProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            threshold: self.threshold,
            window_days: self.window_days,
            limit: self.limit,
            burst: self.burst,
            unit: self.unit.clone(),
            locale: self.locale.clone(),
            budget: self.budget,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a plan proposal in status idle.
pub fn new_plan_proposal(code: &str, title: &str) -> PlanProposal {
    PlanProposal {
        code: code.to_string(),
        title: title.to_string(),
        description: "".to_string(),
        threshold: 0.0,
        window_days: 0,
        limit: 0,
        burst: 0,
        unit: "".to_string(),
        locale: "".to_string(),
        budget: 0.0,
        port: 0,
        status: PlanProposalStatus::Idle,
    }
}
