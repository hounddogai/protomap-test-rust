use super::*;

use acme_kit::text;

/// Where a federation window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationWindowStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FederationWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FederationWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation window as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationWindow {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub limit: i64,
    pub audience: String,
    pub cost: f64,
    pub archived: bool,
    pub active: bool,
    pub created_at: i64,
    pub notes: String,
    pub principal: String,
    pub status: FederationWindowStatus,
}

impl FederationWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationWindowStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationWindowStatus) -> bool {
        match self.status {
            FederationWindowStatus::Proposed => {
                match next {
                    FederationWindowStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationWindowStatus::Approved => {
                return next == FederationWindowStatus::Withdrawn;
            }
            FederationWindowStatus::Rejected => {
                match next {
                    FederationWindowStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FederationWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation window {} cannot move from {} to {}",
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
        format!("{} {}", "federation window", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.created_at as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationWindow {
        FederationWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            limit: self.limit,
            audience: self.audience.clone(),
            cost: self.cost,
            archived: self.archived,
            active: self.active,
            created_at: self.created_at,
            notes: self.notes.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a federation window in status proposed.
pub fn new_federation_window(code: &str, name: &str) -> FederationWindow {
    FederationWindow {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        limit: 0,
        audience: "".to_string(),
        cost: 0.0,
        archived: false,
        active: false,
        created_at: 0,
        notes: "".to_string(),
        principal: "".to_string(),
        status: FederationWindowStatus::Proposed,
    }
}
