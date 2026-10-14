use super::*;

use acme_kit::text;

/// Where a peering tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringTierStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PeeringTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PeeringTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering tier as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringTier {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub audience: String,
    pub starts_at: i64,
    pub approved: bool,
    pub quota_units: i64,
    pub status: PeeringTierStatus,
}

impl PeeringTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringTierStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringTierStatus) -> bool {
        match self.status {
            PeeringTierStatus::Created => {
                match next {
                    PeeringTierStatus::Published | PeeringTierStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringTierStatus::Validated => {
                match next {
                    PeeringTierStatus::Published | PeeringTierStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringTierStatus::Published => {
                match next {
                    PeeringTierStatus::Retired => {
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

    pub fn move_to(&mut self, next: PeeringTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering tier {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.quota_units as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringTier {
        PeeringTier {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            audience: self.audience.clone(),
            starts_at: self.starts_at,
            approved: self.approved,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a peering tier in status created.
pub fn new_peering_tier(code: &str, label: &str) -> PeeringTier {
    PeeringTier {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        audience: "".to_string(),
        starts_at: 0,
        approved: false,
        quota_units: 0,
        status: PeeringTierStatus::Created,
    }
}
