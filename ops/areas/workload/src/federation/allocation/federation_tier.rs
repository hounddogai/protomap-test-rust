use super::*;

use acme_kit::text;

/// Where a federation tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationTierStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FederationTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FederationTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationTier {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub locale: String,
    pub starts_at: i64,
    pub status: FederationTierStatus,
}

impl FederationTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationTierStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationTierStatus) -> bool {
        match self.status {
            FederationTierStatus::Proposed => {
                return next == FederationTierStatus::Rejected;
            }
            FederationTierStatus::Approved => {
                match next {
                    FederationTierStatus::Rejected | FederationTierStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationTierStatus::Rejected => {
                match next {
                    FederationTierStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FederationTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation tier {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.starts_at as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationTier {
        FederationTier {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            locale: self.locale.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a federation tier in status proposed.
pub fn new_federation_tier(code: &str, name: &str) -> FederationTier {
    FederationTier {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        locale: "".to_string(),
        starts_at: 0,
        status: FederationTierStatus::Proposed,
    }
}
