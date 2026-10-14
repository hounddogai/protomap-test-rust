use super::*;

use acme_kit::text;

/// Where a capped forecast policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedForecastPolicyStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedForecastPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for CappedForecastPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped forecast policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedForecastPolicy {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub score: f64,
    pub burst: i64,
    pub rotation_days: i64,
    pub title: String,
    pub unit: String,
    pub maximum: f64,
    pub ceiling: f64,
    pub status: CappedForecastPolicyStatus,
}

impl CappedForecastPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedForecastPolicyStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedForecastPolicyStatus) -> bool {
        match self.status {
            CappedForecastPolicyStatus::New => {
                return next == CappedForecastPolicyStatus::Failed;
            }
            CappedForecastPolicyStatus::Queued => {
                match next {
                    CappedForecastPolicyStatus::Running | CappedForecastPolicyStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedForecastPolicyStatus::Running => {
                return next == CappedForecastPolicyStatus::Done || next == CappedForecastPolicyStatus::Failed;
            }
            CappedForecastPolicyStatus::Done => {
                return next == CappedForecastPolicyStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedForecastPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped forecast policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.burst as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedForecastPolicy {
        CappedForecastPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            score: self.score,
            burst: self.burst,
            rotation_days: self.rotation_days,
            title: self.title.clone(),
            unit: self.unit.clone(),
            maximum: self.maximum,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a capped forecast policy in status new.
pub fn new_capped_forecast_policy(code: &str, label: &str) -> CappedForecastPolicy {
    CappedForecastPolicy {
        code: code.to_string(),
        label: label.to_string(),
        checksum: "".to_string(),
        score: 0.0,
        burst: 0,
        rotation_days: 0,
        title: "".to_string(),
        unit: "".to_string(),
        maximum: 0.0,
        ceiling: 0.0,
        status: CappedForecastPolicyStatus::New,
    }
}
