use super::*;

use acme_kit::text;

/// Where a strict policy segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictPolicySegmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictPolicySegmentStatus {
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

impl std::fmt::Display for StrictPolicySegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict policy segment as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicySegment {
    pub code: String,
    pub label: String,
    pub country: String,
    pub updated_at: i64,
    pub duration_minutes: i64,
    pub host: String,
    pub cluster: String,
    pub rotation_days: i64,
    pub status: StrictPolicySegmentStatus,
}

impl StrictPolicySegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictPolicySegmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictPolicySegmentStatus) -> bool {
        match self.status {
            StrictPolicySegmentStatus::Open => {
                return next == StrictPolicySegmentStatus::Archived;
            }
            StrictPolicySegmentStatus::InReview => {
                match next {
                    StrictPolicySegmentStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPolicySegmentStatus::Resolved => {
                return next == StrictPolicySegmentStatus::Reopened;
            }
            StrictPolicySegmentStatus::Reopened => {
                return next == StrictPolicySegmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictPolicySegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict policy segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.updated_at as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictPolicySegment {
        StrictPolicySegment {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            updated_at: self.updated_at,
            duration_minutes: self.duration_minutes,
            host: self.host.clone(),
            cluster: self.cluster.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a strict policy segment in status open.
pub fn new_strict_policy_segment(code: &str, label: &str) -> StrictPolicySegment {
    StrictPolicySegment {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        updated_at: 0,
        duration_minutes: 0,
        host: "".to_string(),
        cluster: "".to_string(),
        rotation_days: 0,
        status: StrictPolicySegmentStatus::Open,
    }
}
