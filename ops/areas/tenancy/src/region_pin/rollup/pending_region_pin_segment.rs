use super::*;

use acme_kit::text;

/// Where a pending region pin segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRegionPinSegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingRegionPinSegmentStatus {
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

impl std::fmt::Display for PendingRegionPinSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending region pin segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinSegment {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub reviewer: String,
    pub retries: i64,
    pub contact_email: String,
    pub timezone: String,
    pub discount: f64,
    pub tenant_id: String,
    pub currency: String,
    pub created_at: i64,
    pub status: PendingRegionPinSegmentStatus,
}

impl PendingRegionPinSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRegionPinSegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRegionPinSegmentStatus) -> bool {
        match self.status {
            PendingRegionPinSegmentStatus::Idle => {
                return next == PendingRegionPinSegmentStatus::Warming;
            }
            PendingRegionPinSegmentStatus::Warming => {
                return next == PendingRegionPinSegmentStatus::Ready || next == PendingRegionPinSegmentStatus::Draining;
            }
            PendingRegionPinSegmentStatus::Ready => {
                match next {
                    PendingRegionPinSegmentStatus::Draining | PendingRegionPinSegmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRegionPinSegmentStatus::Draining => {
                return next == PendingRegionPinSegmentStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRegionPinSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending region pin segment {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.retries as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRegionPinSegment {
        PendingRegionPinSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            reviewer: self.reviewer.clone(),
            retries: self.retries,
            contact_email: self.contact_email.clone(),
            timezone: self.timezone.clone(),
            discount: self.discount,
            tenant_id: self.tenant_id.clone(),
            currency: self.currency.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending region pin segment in status idle.
pub fn new_pending_region_pin_segment(code: &str, label: &str) -> PendingRegionPinSegment {
    PendingRegionPinSegment {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        reviewer: "".to_string(),
        retries: 0,
        contact_email: "".to_string(),
        timezone: "".to_string(),
        discount: 0.0,
        tenant_id: "".to_string(),
        currency: "".to_string(),
        created_at: 0,
        status: PendingRegionPinSegmentStatus::Idle,
    }
}
