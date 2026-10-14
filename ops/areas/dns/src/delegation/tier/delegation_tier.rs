use super::*;

use acme_kit::text;

/// Where a delegation tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationTierStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DelegationTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DelegationTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A delegation tier as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTier {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub owner_id: String,
    pub created_at: i64,
    pub priority: i64,
    pub category: String,
    pub external_ref: String,
    pub ratio: f64,
    pub floor_value: f64,
    pub description: String,
    pub status: DelegationTierStatus,
}

impl DelegationTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationTierStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationTierStatus) -> bool {
        match self.status {
            DelegationTierStatus::Proposed => {
                match next {
                    DelegationTierStatus::Rejected | DelegationTierStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationTierStatus::Approved => {
                return next == DelegationTierStatus::Rejected;
            }
            DelegationTierStatus::Rejected => {
                match next {
                    DelegationTierStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DelegationTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation tier {} cannot move from {} to {}",
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

    pub fn weighted_value(&self) -> f64 {
        ((self.sequence as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationTier {
        DelegationTier {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            owner_id: self.owner_id.clone(),
            created_at: self.created_at,
            priority: self.priority,
            category: self.category.clone(),
            external_ref: self.external_ref.clone(),
            ratio: self.ratio,
            floor_value: self.floor_value,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a delegation tier in status proposed.
pub fn new_delegation_tier(code: &str, name: &str) -> DelegationTier {
    DelegationTier {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        owner_id: "".to_string(),
        created_at: 0,
        priority: 0,
        category: "".to_string(),
        external_ref: "".to_string(),
        ratio: 0.0,
        floor_value: 0.0,
        description: "".to_string(),
        status: DelegationTierStatus::Proposed,
    }
}
