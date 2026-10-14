use super::*;

use acme_kit::text;

/// Where a global peering threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPeeringThresholdStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalPeeringThresholdStatus {
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

impl std::fmt::Display for GlobalPeeringThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global peering threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringThreshold {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub ratio: f64,
    pub burst: i64,
    pub retries: i64,
    pub port: i64,
    pub pinned: bool,
    pub phone: String,
    pub status: GlobalPeeringThresholdStatus,
}

impl GlobalPeeringThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPeeringThresholdStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPeeringThresholdStatus) -> bool {
        match self.status {
            GlobalPeeringThresholdStatus::Draft => {
                return next == GlobalPeeringThresholdStatus::Pending || next == GlobalPeeringThresholdStatus::Closed;
            }
            GlobalPeeringThresholdStatus::Pending => {
                return next == GlobalPeeringThresholdStatus::Closed;
            }
            GlobalPeeringThresholdStatus::Active => {
                return next == GlobalPeeringThresholdStatus::Suspended || next == GlobalPeeringThresholdStatus::Closed;
            }
            GlobalPeeringThresholdStatus::Suspended => {
                return next == GlobalPeeringThresholdStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalPeeringThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global peering threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.burst as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalPeeringThreshold {
        GlobalPeeringThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            ratio: self.ratio,
            burst: self.burst,
            retries: self.retries,
            port: self.port,
            pinned: self.pinned,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global peering threshold in status draft.
pub fn new_global_peering_threshold(code: &str, name: &str) -> GlobalPeeringThreshold {
    GlobalPeeringThreshold {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        ratio: 0.0,
        burst: 0,
        retries: 0,
        port: 0,
        pinned: false,
        phone: "".to_string(),
        status: GlobalPeeringThresholdStatus::Draft,
    }
}
