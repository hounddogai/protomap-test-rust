use super::*;

use acme_kit::text;

/// Where a rolling probe mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingProbeMappingStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingProbeMappingStatus {
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

impl std::fmt::Display for RollingProbeMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling probe mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeMapping {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub quantity: i64,
    pub ends_at: i64,
    pub sequence: i64,
    pub fingerprint: String,
    pub archived: bool,
    pub ratio: f64,
    pub status: RollingProbeMappingStatus,
}

impl RollingProbeMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingProbeMappingStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingProbeMappingStatus) -> bool {
        match self.status {
            RollingProbeMappingStatus::Open => {
                match next {
                    RollingProbeMappingStatus::Resolved | RollingProbeMappingStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingProbeMappingStatus::InReview => {
                return next == RollingProbeMappingStatus::Resolved || next == RollingProbeMappingStatus::Archived;
            }
            RollingProbeMappingStatus::Resolved => {
                match next {
                    RollingProbeMappingStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingProbeMappingStatus::Reopened => {
                return next == RollingProbeMappingStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingProbeMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling probe mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling probe mapping", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.sequence as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingProbeMapping {
        RollingProbeMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            quantity: self.quantity,
            ends_at: self.ends_at,
            sequence: self.sequence,
            fingerprint: self.fingerprint.clone(),
            archived: self.archived,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rolling probe mapping in status open.
pub fn new_rolling_probe_mapping(code: &str, title: &str) -> RollingProbeMapping {
    RollingProbeMapping {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        quantity: 0,
        ends_at: 0,
        sequence: 0,
        fingerprint: "".to_string(),
        archived: false,
        ratio: 0.0,
        status: RollingProbeMappingStatus::Open,
    }
}
