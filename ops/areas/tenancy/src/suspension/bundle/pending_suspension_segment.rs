use super::*;

use acme_kit::text;

/// Where a pending suspension segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingSuspensionSegmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingSuspensionSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingSuspensionSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending suspension segment as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSuspensionSegment {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub version: i64,
    pub phone: String,
    pub minimum: f64,
    pub ends_at: i64,
    pub threshold: f64,
    pub owner_id: String,
    pub ttl_seconds: i64,
    pub status: PendingSuspensionSegmentStatus,
}

impl PendingSuspensionSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingSuspensionSegmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingSuspensionSegmentStatus) -> bool {
        match self.status {
            PendingSuspensionSegmentStatus::Proposed => {
                return next == PendingSuspensionSegmentStatus::Approved;
            }
            PendingSuspensionSegmentStatus::Approved => {
                return next == PendingSuspensionSegmentStatus::Withdrawn;
            }
            PendingSuspensionSegmentStatus::Rejected => {
                return next == PendingSuspensionSegmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingSuspensionSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending suspension segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending suspension segment", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.minimum * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingSuspensionSegment {
        PendingSuspensionSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            version: self.version,
            phone: self.phone.clone(),
            minimum: self.minimum,
            ends_at: self.ends_at,
            threshold: self.threshold,
            owner_id: self.owner_id.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending suspension segment in status proposed.
pub fn new_pending_suspension_segment(code: &str, title: &str) -> PendingSuspensionSegment {
    PendingSuspensionSegment {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        version: 0,
        phone: "".to_string(),
        minimum: 0.0,
        ends_at: 0,
        threshold: 0.0,
        owner_id: "".to_string(),
        ttl_seconds: 0,
        status: PendingSuspensionSegmentStatus::Proposed,
    }
}
