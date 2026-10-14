use super::*;

use acme_kit::text;

/// Where a segment plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentPlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SegmentPlanStatus {
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

impl std::fmt::Display for SegmentPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPlan {
    pub code: String,
    pub title: String,
    pub country: String,
    pub region: String,
    pub channel: String,
    pub principal: String,
    pub attempts: i64,
    pub status: SegmentPlanStatus,
}

impl SegmentPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentPlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentPlanStatus) -> bool {
        match self.status {
            SegmentPlanStatus::Open => {
                return next == SegmentPlanStatus::Resolved || next == SegmentPlanStatus::Reopened;
            }
            SegmentPlanStatus::InReview => {
                match next {
                    SegmentPlanStatus::Resolved | SegmentPlanStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentPlanStatus::Resolved => {
                match next {
                    SegmentPlanStatus::Reopened | SegmentPlanStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentPlanStatus::Reopened => {
                match next {
                    SegmentPlanStatus::Archived => {
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

    pub fn move_to(&mut self, next: SegmentPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.attempts as f64
    }

    pub fn copy(&self) -> SegmentPlan {
        SegmentPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            region: self.region.clone(),
            channel: self.channel.clone(),
            principal: self.principal.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a segment plan in status open.
pub fn new_segment_plan(code: &str, title: &str) -> SegmentPlan {
    SegmentPlan {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        region: "".to_string(),
        channel: "".to_string(),
        principal: "".to_string(),
        attempts: 0,
        status: SegmentPlanStatus::Open,
    }
}
