use super::*;

use acme_kit::text;

/// Where a flow log estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogEstimateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FlowLogEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for FlowLogEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One flow log estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogEstimate {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub score: f64,
    pub amount: f64,
    pub status: FlowLogEstimateStatus,
}

impl FlowLogEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogEstimateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogEstimateStatus) -> bool {
        match self.status {
            FlowLogEstimateStatus::Planned => {
                return next == FlowLogEstimateStatus::Scheduled || next == FlowLogEstimateStatus::Completed;
            }
            FlowLogEstimateStatus::Scheduled => {
                match next {
                    FlowLogEstimateStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogEstimateStatus::InProgress => {
                return next == FlowLogEstimateStatus::Completed;
            }
            FlowLogEstimateStatus::Completed => {
                return next == FlowLogEstimateStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FlowLogEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "flow log estimate", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.version as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogEstimate {
        FlowLogEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            score: self.score,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a flow log estimate in status planned.
pub fn new_flow_log_estimate(code: &str, label: &str) -> FlowLogEstimate {
    FlowLogEstimate {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        score: 0.0,
        amount: 0.0,
        status: FlowLogEstimateStatus::Planned,
    }
}
