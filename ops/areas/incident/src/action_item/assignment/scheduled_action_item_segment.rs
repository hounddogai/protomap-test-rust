use super::*;

use acme_kit::text;

/// Where a scheduled action item segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledActionItemSegmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledActionItemSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledActionItemSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled action item segment as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledActionItemSegment {
    pub code: String,
    pub label: String,
    pub country: String,
    pub burst: i64,
    pub maximum: f64,
    pub currency: String,
    pub issuer: String,
    pub status: ScheduledActionItemSegmentStatus,
}

impl ScheduledActionItemSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledActionItemSegmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledActionItemSegmentStatus) -> bool {
        match self.status {
            ScheduledActionItemSegmentStatus::Requested => {
                match next {
                    ScheduledActionItemSegmentStatus::Granted | ScheduledActionItemSegmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledActionItemSegmentStatus::Granted => {
                return next == ScheduledActionItemSegmentStatus::Revoked;
            }
            ScheduledActionItemSegmentStatus::Revoked => {
                return next == ScheduledActionItemSegmentStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledActionItemSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled action item segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.maximum * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledActionItemSegment {
        ScheduledActionItemSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            burst: self.burst,
            maximum: self.maximum,
            currency: self.currency.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a scheduled action item segment in status requested.
pub fn new_scheduled_action_item_segment(code: &str, label: &str) -> ScheduledActionItemSegment {
    ScheduledActionItemSegment {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        burst: 0,
        maximum: 0.0,
        currency: "".to_string(),
        issuer: "".to_string(),
        status: ScheduledActionItemSegmentStatus::Requested,
    }
}
