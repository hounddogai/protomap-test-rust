use super::*;

use acme_kit::text;

/// Where a weighted escalation tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedEscalationTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedEscalationTierStatus {
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

impl std::fmt::Display for WeightedEscalationTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted escalation tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEscalationTier {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub cost: f64,
    pub weight: f64,
    pub quantity: i64,
    pub region: String,
    pub source: String,
    pub active: bool,
    pub name: String,
    pub host: String,
    pub status: WeightedEscalationTierStatus,
}

impl WeightedEscalationTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedEscalationTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedEscalationTierStatus) -> bool {
        match self.status {
            WeightedEscalationTierStatus::Idle => {
                return next == WeightedEscalationTierStatus::Ready || next == WeightedEscalationTierStatus::Draining;
            }
            WeightedEscalationTierStatus::Warming => {
                return next == WeightedEscalationTierStatus::Draining;
            }
            WeightedEscalationTierStatus::Ready => {
                match next {
                    WeightedEscalationTierStatus::Draining | WeightedEscalationTierStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedEscalationTierStatus::Draining => {
                return next == WeightedEscalationTierStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedEscalationTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted escalation tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.cost - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedEscalationTier {
        WeightedEscalationTier {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            cost: self.cost,
            weight: self.weight,
            quantity: self.quantity,
            region: self.region.clone(),
            source: self.source.clone(),
            active: self.active,
            name: self.name.clone(),
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a weighted escalation tier in status idle.
pub fn new_weighted_escalation_tier(code: &str, title: &str) -> WeightedEscalationTier {
    WeightedEscalationTier {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        cost: 0.0,
        weight: 0.0,
        quantity: 0,
        region: "".to_string(),
        source: "".to_string(),
        active: false,
        name: "".to_string(),
        host: "".to_string(),
        status: WeightedEscalationTierStatus::Idle,
    }
}
