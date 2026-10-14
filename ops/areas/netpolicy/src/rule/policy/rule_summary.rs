use super::*;

use acme_kit::text;

/// Where a rule summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleSummaryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RuleSummaryStatus {
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

impl std::fmt::Display for RuleSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rule summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleSummary {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub region: String,
    pub approved: bool,
    pub reviewer: String,
    pub capacity: i64,
    pub description: String,
    pub unit: String,
    pub status: RuleSummaryStatus,
}

impl RuleSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleSummaryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleSummaryStatus) -> bool {
        match self.status {
            RuleSummaryStatus::Idle => {
                match next {
                    RuleSummaryStatus::Warming | RuleSummaryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleSummaryStatus::Warming => {
                match next {
                    RuleSummaryStatus::Draining | RuleSummaryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleSummaryStatus::Ready => {
                match next {
                    RuleSummaryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleSummaryStatus::Draining => {
                return next == RuleSummaryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rule summary", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.capacity as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleSummary {
        RuleSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            region: self.region.clone(),
            approved: self.approved,
            reviewer: self.reviewer.clone(),
            capacity: self.capacity,
            description: self.description.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a rule summary in status idle.
pub fn new_rule_summary(code: &str, label: &str) -> RuleSummary {
    RuleSummary {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        region: "".to_string(),
        approved: false,
        reviewer: "".to_string(),
        capacity: 0,
        description: "".to_string(),
        unit: "".to_string(),
        status: RuleSummaryStatus::Idle,
    }
}
