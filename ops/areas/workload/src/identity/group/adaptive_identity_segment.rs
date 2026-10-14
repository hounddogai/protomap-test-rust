use super::*;

use acme_kit::text;

/// Where a adaptive identity segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveIdentitySegmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveIdentitySegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveIdentitySegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive identity segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentitySegment {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub port: i64,
    pub budget: f64,
    pub owner_id: String,
    pub priority: i64,
    pub status: AdaptiveIdentitySegmentStatus,
}

impl AdaptiveIdentitySegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveIdentitySegmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveIdentitySegmentStatus) -> bool {
        match self.status {
            AdaptiveIdentitySegmentStatus::Proposed => {
                return next == AdaptiveIdentitySegmentStatus::Approved;
            }
            AdaptiveIdentitySegmentStatus::Approved => {
                match next {
                    AdaptiveIdentitySegmentStatus::Rejected | AdaptiveIdentitySegmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveIdentitySegmentStatus::Rejected => {
                match next {
                    AdaptiveIdentitySegmentStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AdaptiveIdentitySegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive identity segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive identity segment", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.port as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveIdentitySegment {
        AdaptiveIdentitySegment {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            port: self.port,
            budget: self.budget,
            owner_id: self.owner_id.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a adaptive identity segment in status proposed.
pub fn new_adaptive_identity_segment(code: &str, label: &str) -> AdaptiveIdentitySegment {
    AdaptiveIdentitySegment {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        port: 0,
        budget: 0.0,
        owner_id: "".to_string(),
        priority: 0,
        status: AdaptiveIdentitySegmentStatus::Proposed,
    }
}
