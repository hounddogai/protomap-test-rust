use super::*;

use acme_kit::text;

/// Where a dashboard checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardCheckpointStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DashboardCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DashboardCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One dashboard checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardCheckpoint {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub contact_email: String,
    pub cipher: String,
    pub budget: f64,
    pub rotation_days: i64,
    pub phone: String,
    pub source: String,
    pub status: DashboardCheckpointStatus,
}

impl DashboardCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardCheckpointStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardCheckpointStatus) -> bool {
        match self.status {
            DashboardCheckpointStatus::Created => {
                match next {
                    DashboardCheckpointStatus::Validated | DashboardCheckpointStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardCheckpointStatus::Validated => {
                match next {
                    DashboardCheckpointStatus::Published | DashboardCheckpointStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardCheckpointStatus::Published => {
                match next {
                    DashboardCheckpointStatus::Retired => {
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

    pub fn move_to(&mut self, next: DashboardCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.rotation_days as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardCheckpoint {
        DashboardCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            contact_email: self.contact_email.clone(),
            cipher: self.cipher.clone(),
            budget: self.budget,
            rotation_days: self.rotation_days,
            phone: self.phone.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a dashboard checkpoint in status created.
pub fn new_dashboard_checkpoint(code: &str, name: &str) -> DashboardCheckpoint {
    DashboardCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        contact_email: "".to_string(),
        cipher: "".to_string(),
        budget: 0.0,
        rotation_days: 0,
        phone: "".to_string(),
        source: "".to_string(),
        status: DashboardCheckpointStatus::Created,
    }
}
