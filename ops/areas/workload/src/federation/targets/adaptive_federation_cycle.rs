use super::*;

use acme_kit::text;

/// Where a adaptive federation cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveFederationCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveFederationCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveFederationCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive federation cycle as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFederationCycle {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub channel: String,
    pub limit: i64,
    pub timezone: String,
    pub owner_id: String,
    pub ceiling: f64,
    pub updated_at: i64,
    pub status: AdaptiveFederationCycleStatus,
}

impl AdaptiveFederationCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveFederationCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveFederationCycleStatus) -> bool {
        match self.status {
            AdaptiveFederationCycleStatus::Proposed => {
                match next {
                    AdaptiveFederationCycleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveFederationCycleStatus::Approved => {
                match next {
                    AdaptiveFederationCycleStatus::Rejected | AdaptiveFederationCycleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveFederationCycleStatus::Rejected => {
                return next == AdaptiveFederationCycleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveFederationCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive federation cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive federation cycle", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.updated_at as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveFederationCycle {
        AdaptiveFederationCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            channel: self.channel.clone(),
            limit: self.limit,
            timezone: self.timezone.clone(),
            owner_id: self.owner_id.clone(),
            ceiling: self.ceiling,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a adaptive federation cycle in status proposed.
pub fn new_adaptive_federation_cycle(code: &str, label: &str) -> AdaptiveFederationCycle {
    AdaptiveFederationCycle {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        channel: "".to_string(),
        limit: 0,
        timezone: "".to_string(),
        owner_id: "".to_string(),
        ceiling: 0.0,
        updated_at: 0,
        status: AdaptiveFederationCycleStatus::Proposed,
    }
}
