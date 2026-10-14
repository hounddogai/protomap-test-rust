use super::*;

use acme_kit::text;

/// Where a release segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseSegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ReleaseSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ReleaseSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSegment {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub burst: i64,
    pub expires_at: i64,
    pub source: String,
    pub starts_at: i64,
    pub key_id: String,
    pub quota_units: i64,
    pub segment: String,
    pub status: ReleaseSegmentStatus,
}

impl ReleaseSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseSegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseSegmentStatus) -> bool {
        match self.status {
            ReleaseSegmentStatus::Idle => {
                return next == ReleaseSegmentStatus::Draining;
            }
            ReleaseSegmentStatus::Warming => {
                return next == ReleaseSegmentStatus::Stopped;
            }
            ReleaseSegmentStatus::Ready => {
                return next == ReleaseSegmentStatus::Draining || next == ReleaseSegmentStatus::Stopped;
            }
            ReleaseSegmentStatus::Draining => {
                match next {
                    ReleaseSegmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ReleaseSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.expires_at as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseSegment {
        ReleaseSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            burst: self.burst,
            expires_at: self.expires_at,
            source: self.source.clone(),
            starts_at: self.starts_at,
            key_id: self.key_id.clone(),
            quota_units: self.quota_units,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a release segment in status idle.
pub fn new_release_segment(code: &str, name: &str) -> ReleaseSegment {
    ReleaseSegment {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        burst: 0,
        expires_at: 0,
        source: "".to_string(),
        starts_at: 0,
        key_id: "".to_string(),
        quota_units: 0,
        segment: "".to_string(),
        status: ReleaseSegmentStatus::Idle,
    }
}
