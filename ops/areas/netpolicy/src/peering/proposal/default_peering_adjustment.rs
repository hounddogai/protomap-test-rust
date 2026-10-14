use super::*;

use acme_kit::text;

/// Where a default peering adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultPeeringAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultPeeringAdjustmentStatus {
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

impl std::fmt::Display for DefaultPeeringAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default peering adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringAdjustment {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub quantity: i64,
    pub ceiling: f64,
    pub floor_value: f64,
    pub retries: i64,
    pub region: String,
    pub fingerprint: String,
    pub name: String,
    pub status: DefaultPeeringAdjustmentStatus,
}

impl DefaultPeeringAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultPeeringAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultPeeringAdjustmentStatus) -> bool {
        match self.status {
            DefaultPeeringAdjustmentStatus::New => {
                match next {
                    DefaultPeeringAdjustmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultPeeringAdjustmentStatus::Queued => {
                return next == DefaultPeeringAdjustmentStatus::Done || next == DefaultPeeringAdjustmentStatus::Failed;
            }
            DefaultPeeringAdjustmentStatus::Running => {
                return next == DefaultPeeringAdjustmentStatus::Done || next == DefaultPeeringAdjustmentStatus::Failed;
            }
            DefaultPeeringAdjustmentStatus::Done => {
                return next == DefaultPeeringAdjustmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultPeeringAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default peering adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.quantity as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultPeeringAdjustment {
        DefaultPeeringAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            quantity: self.quantity,
            ceiling: self.ceiling,
            floor_value: self.floor_value,
            retries: self.retries,
            region: self.region.clone(),
            fingerprint: self.fingerprint.clone(),
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a default peering adjustment in status new.
pub fn new_default_peering_adjustment(code: &str, label: &str) -> DefaultPeeringAdjustment {
    DefaultPeeringAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        quantity: 0,
        ceiling: 0.0,
        floor_value: 0.0,
        retries: 0,
        region: "".to_string(),
        fingerprint: "".to_string(),
        name: "".to_string(),
        status: DefaultPeeringAdjustmentStatus::New,
    }
}
