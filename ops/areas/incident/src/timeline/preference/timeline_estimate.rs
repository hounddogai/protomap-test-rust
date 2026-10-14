use super::*;

use acme_kit::text;

/// Where a timeline estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TimelineEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TimelineEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline estimate as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineEstimate {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub tenant_id: String,
    pub ttl_seconds: i64,
    pub description: String,
    pub name: String,
    pub category: String,
    pub pinned: bool,
    pub status: TimelineEstimateStatus,
}

impl TimelineEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineEstimateStatus) -> bool {
        match self.status {
            TimelineEstimateStatus::Created => {
                return next == TimelineEstimateStatus::Validated || next == TimelineEstimateStatus::Retired;
            }
            TimelineEstimateStatus::Validated => {
                return next == TimelineEstimateStatus::Published || next == TimelineEstimateStatus::Retired;
            }
            TimelineEstimateStatus::Published => {
                return next == TimelineEstimateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TimelineEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> TimelineEstimate {
        TimelineEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            tenant_id: self.tenant_id.clone(),
            ttl_seconds: self.ttl_seconds,
            description: self.description.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a timeline estimate in status created.
pub fn new_timeline_estimate(code: &str, title: &str) -> TimelineEstimate {
    TimelineEstimate {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        tenant_id: "".to_string(),
        ttl_seconds: 0,
        description: "".to_string(),
        name: "".to_string(),
        category: "".to_string(),
        pinned: false,
        status: TimelineEstimateStatus::Created,
    }
}
