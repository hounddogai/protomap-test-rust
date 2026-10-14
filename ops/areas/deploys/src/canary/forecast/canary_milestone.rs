use super::*;

use acme_kit::text;

/// Where a canary milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryMilestoneStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CanaryMilestoneStatus {
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

impl std::fmt::Display for CanaryMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A canary milestone as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryMilestone {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub maximum: f64,
    pub limit: i64,
    pub burst: i64,
    pub summary: String,
    pub minimum: f64,
    pub margin: f64,
    pub segment: String,
    pub status: CanaryMilestoneStatus,
}

impl CanaryMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryMilestoneStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryMilestoneStatus) -> bool {
        match self.status {
            CanaryMilestoneStatus::Open => {
                return next == CanaryMilestoneStatus::Archived;
            }
            CanaryMilestoneStatus::InReview => {
                return next == CanaryMilestoneStatus::Resolved || next == CanaryMilestoneStatus::Archived;
            }
            CanaryMilestoneStatus::Resolved => {
                return next == CanaryMilestoneStatus::Reopened || next == CanaryMilestoneStatus::Archived;
            }
            CanaryMilestoneStatus::Reopened => {
                match next {
                    CanaryMilestoneStatus::Archived => {
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

    pub fn move_to(&mut self, next: CanaryMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        (self.margin * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryMilestone {
        CanaryMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            maximum: self.maximum,
            limit: self.limit,
            burst: self.burst,
            summary: self.summary.clone(),
            minimum: self.minimum,
            margin: self.margin,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a canary milestone in status open.
pub fn new_canary_milestone(code: &str, title: &str) -> CanaryMilestone {
    CanaryMilestone {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        maximum: 0.0,
        limit: 0,
        burst: 0,
        summary: "".to_string(),
        minimum: 0.0,
        margin: 0.0,
        segment: "".to_string(),
        status: CanaryMilestoneStatus::Open,
    }
}
