use super::*;

use acme_kit::text;

/// Where a legacy egress rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyEgressRollupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyEgressRollupStatus {
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

impl std::fmt::Display for LegacyEgressRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy egress rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEgressRollup {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub priority: i64,
    pub quantity: i64,
    pub region: String,
    pub label: String,
    pub cluster: String,
    pub starts_at: i64,
    pub status: LegacyEgressRollupStatus,
}

impl LegacyEgressRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyEgressRollupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyEgressRollupStatus) -> bool {
        match self.status {
            LegacyEgressRollupStatus::Idle => {
                match next {
                    LegacyEgressRollupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyEgressRollupStatus::Warming => {
                return next == LegacyEgressRollupStatus::Ready || next == LegacyEgressRollupStatus::Stopped;
            }
            LegacyEgressRollupStatus::Ready => {
                match next {
                    LegacyEgressRollupStatus::Draining | LegacyEgressRollupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyEgressRollupStatus::Draining => {
                return next == LegacyEgressRollupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyEgressRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy egress rollup {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.starts_at as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyEgressRollup {
        LegacyEgressRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            priority: self.priority,
            quantity: self.quantity,
            region: self.region.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a legacy egress rollup in status idle.
pub fn new_legacy_egress_rollup(code: &str, name: &str) -> LegacyEgressRollup {
    LegacyEgressRollup {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        priority: 0,
        quantity: 0,
        region: "".to_string(),
        label: "".to_string(),
        cluster: "".to_string(),
        starts_at: 0,
        status: LegacyEgressRollupStatus::Idle,
    }
}
