use super::*;

use acme_kit::text;

/// Where a fallback zone threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackZoneThresholdStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackZoneThresholdStatus {
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

impl std::fmt::Display for FallbackZoneThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback zone threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneThreshold {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub key_id: String,
    pub summary: String,
    pub quantity: i64,
    pub sequence: i64,
    pub audience: String,
    pub ends_at: i64,
    pub archived: bool,
    pub floor_value: f64,
    pub status: FallbackZoneThresholdStatus,
}

impl FallbackZoneThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackZoneThresholdStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackZoneThresholdStatus) -> bool {
        match self.status {
            FallbackZoneThresholdStatus::New => {
                match next {
                    FallbackZoneThresholdStatus::Queued | FallbackZoneThresholdStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackZoneThresholdStatus::Queued => {
                return next == FallbackZoneThresholdStatus::Done || next == FallbackZoneThresholdStatus::Failed;
            }
            FallbackZoneThresholdStatus::Running => {
                return next == FallbackZoneThresholdStatus::Failed;
            }
            FallbackZoneThresholdStatus::Done => {
                return next == FallbackZoneThresholdStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackZoneThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback zone threshold {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ends_at as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackZoneThreshold {
        FallbackZoneThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            key_id: self.key_id.clone(),
            summary: self.summary.clone(),
            quantity: self.quantity,
            sequence: self.sequence,
            audience: self.audience.clone(),
            ends_at: self.ends_at,
            archived: self.archived,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a fallback zone threshold in status new.
pub fn new_fallback_zone_threshold(code: &str, name: &str) -> FallbackZoneThreshold {
    FallbackZoneThreshold {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        key_id: "".to_string(),
        summary: "".to_string(),
        quantity: 0,
        sequence: 0,
        audience: "".to_string(),
        ends_at: 0,
        archived: false,
        floor_value: 0.0,
        status: FallbackZoneThresholdStatus::New,
    }
}
