use super::*;

use acme_kit::text;

/// Where a region pin summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinSummaryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionPinSummaryStatus {
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

impl std::fmt::Display for RegionPinSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One region pin summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSummary {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub description: String,
    pub floor_value: f64,
    pub threshold: f64,
    pub score: f64,
    pub status: RegionPinSummaryStatus,
}

impl RegionPinSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinSummaryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinSummaryStatus) -> bool {
        match self.status {
            RegionPinSummaryStatus::Open => {
                return next == RegionPinSummaryStatus::Resolved;
            }
            RegionPinSummaryStatus::InReview => {
                return next == RegionPinSummaryStatus::Resolved || next == RegionPinSummaryStatus::Archived;
            }
            RegionPinSummaryStatus::Resolved => {
                match next {
                    RegionPinSummaryStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinSummaryStatus::Reopened => {
                return next == RegionPinSummaryStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "region pin summary", self.code)
    }

    pub fn net_load(&self) -> f64 {
        (self.floor_value * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinSummary {
        RegionPinSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            description: self.description.clone(),
            floor_value: self.floor_value,
            threshold: self.threshold,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a region pin summary in status open.
pub fn new_region_pin_summary(code: &str, label: &str) -> RegionPinSummary {
    RegionPinSummary {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        description: "".to_string(),
        floor_value: 0.0,
        threshold: 0.0,
        score: 0.0,
        status: RegionPinSummaryStatus::Open,
    }
}
