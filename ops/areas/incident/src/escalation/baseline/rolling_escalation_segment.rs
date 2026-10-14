use super::*;

use acme_kit::text;

/// Where a rolling escalation segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingEscalationSegmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingEscalationSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RollingEscalationSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling escalation segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationSegment {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub fingerprint: String,
    pub host: String,
    pub floor_value: f64,
    pub created_at: i64,
    pub score: f64,
    pub source: String,
    pub status: RollingEscalationSegmentStatus,
}

impl RollingEscalationSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingEscalationSegmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingEscalationSegmentStatus) -> bool {
        match self.status {
            RollingEscalationSegmentStatus::Draft => {
                return next == RollingEscalationSegmentStatus::Active
                    || next == RollingEscalationSegmentStatus::Closed;
            }
            RollingEscalationSegmentStatus::Pending => {
                return next == RollingEscalationSegmentStatus::Closed;
            }
            RollingEscalationSegmentStatus::Active => {
                return next == RollingEscalationSegmentStatus::Suspended
                    || next == RollingEscalationSegmentStatus::Closed;
            }
            RollingEscalationSegmentStatus::Suspended => {
                match next {
                    RollingEscalationSegmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: RollingEscalationSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling escalation segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.created_at as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingEscalationSegment {
        RollingEscalationSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            fingerprint: self.fingerprint.clone(),
            host: self.host.clone(),
            floor_value: self.floor_value,
            created_at: self.created_at,
            score: self.score,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rolling escalation segment in status draft.
pub fn new_rolling_escalation_segment(code: &str, title: &str) -> RollingEscalationSegment {
    RollingEscalationSegment {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        fingerprint: "".to_string(),
        host: "".to_string(),
        floor_value: 0.0,
        created_at: 0,
        score: 0.0,
        source: "".to_string(),
        status: RollingEscalationSegmentStatus::Draft,
    }
}
