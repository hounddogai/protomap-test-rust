use super::*;

use acme_kit::text;

/// Where a failover estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverEstimateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FailoverEstimateStatus {
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

impl std::fmt::Display for FailoverEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a failover estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverEstimate {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub cipher: String,
    pub cost: f64,
    pub region: String,
    pub host: String,
    pub visible: bool,
    pub batch_size: i64,
    pub rate: f64,
    pub audience: String,
    pub status: FailoverEstimateStatus,
}

impl FailoverEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverEstimateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverEstimateStatus) -> bool {
        match self.status {
            FailoverEstimateStatus::Open => {
                match next {
                    FailoverEstimateStatus::Resolved | FailoverEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverEstimateStatus::InReview => {
                match next {
                    FailoverEstimateStatus::Reopened | FailoverEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverEstimateStatus::Resolved => {
                return next == FailoverEstimateStatus::Reopened;
            }
            FailoverEstimateStatus::Reopened => {
                return next == FailoverEstimateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.budget - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverEstimate {
        FailoverEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            cipher: self.cipher.clone(),
            cost: self.cost,
            region: self.region.clone(),
            host: self.host.clone(),
            visible: self.visible,
            batch_size: self.batch_size,
            rate: self.rate,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a failover estimate in status open.
pub fn new_failover_estimate(code: &str, title: &str) -> FailoverEstimate {
    FailoverEstimate {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        cipher: "".to_string(),
        cost: 0.0,
        region: "".to_string(),
        host: "".to_string(),
        visible: false,
        batch_size: 0,
        rate: 0.0,
        audience: "".to_string(),
        status: FailoverEstimateStatus::Open,
    }
}
