use super::*;

use acme_kit::text;

/// Where a segment adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentAdjustmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SegmentAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SegmentAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAdjustment {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub active: bool,
    pub port: i64,
    pub budget: f64,
    pub status: SegmentAdjustmentStatus,
}

impl SegmentAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentAdjustmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentAdjustmentStatus) -> bool {
        match self.status {
            SegmentAdjustmentStatus::Created => {
                return next == SegmentAdjustmentStatus::Validated || next == SegmentAdjustmentStatus::Retired;
            }
            SegmentAdjustmentStatus::Validated => {
                match next {
                    SegmentAdjustmentStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentAdjustmentStatus::Published => {
                return next == SegmentAdjustmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.budget - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentAdjustment {
        SegmentAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            active: self.active,
            port: self.port,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a segment adjustment in status created.
pub fn new_segment_adjustment(code: &str, name: &str) -> SegmentAdjustment {
    SegmentAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        active: false,
        port: 0,
        budget: 0.0,
        status: SegmentAdjustmentStatus::Created,
    }
}
