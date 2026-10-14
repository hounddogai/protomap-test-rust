use super::*;

use acme_kit::text;

/// Where a capped token segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedTokenSegmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedTokenSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for CappedTokenSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped token segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenSegment {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub issuer: String,
    pub principal: String,
    pub quota_units: i64,
    pub segment: String,
    pub ratio: f64,
    pub priority: i64,
    pub country: String,
    pub starts_at: i64,
    pub status: CappedTokenSegmentStatus,
}

impl CappedTokenSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedTokenSegmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedTokenSegmentStatus) -> bool {
        match self.status {
            CappedTokenSegmentStatus::New => {
                match next {
                    CappedTokenSegmentStatus::Queued | CappedTokenSegmentStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedTokenSegmentStatus::Queued => {
                return next == CappedTokenSegmentStatus::Done;
            }
            CappedTokenSegmentStatus::Running => {
                return next == CappedTokenSegmentStatus::Done;
            }
            CappedTokenSegmentStatus::Done => {
                return next == CappedTokenSegmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedTokenSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped token segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.ratio + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedTokenSegment {
        CappedTokenSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            issuer: self.issuer.clone(),
            principal: self.principal.clone(),
            quota_units: self.quota_units,
            segment: self.segment.clone(),
            ratio: self.ratio,
            priority: self.priority,
            country: self.country.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a capped token segment in status new.
pub fn new_capped_token_segment(code: &str, title: &str) -> CappedTokenSegment {
    CappedTokenSegment {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        issuer: "".to_string(),
        principal: "".to_string(),
        quota_units: 0,
        segment: "".to_string(),
        ratio: 0.0,
        priority: 0,
        country: "".to_string(),
        starts_at: 0,
        status: CappedTokenSegmentStatus::New,
    }
}
