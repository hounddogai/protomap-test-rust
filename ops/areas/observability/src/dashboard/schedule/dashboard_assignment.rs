use super::*;

use acme_kit::text;

/// Where a dashboard assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DashboardAssignmentStatus {
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

impl std::fmt::Display for DashboardAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dashboard assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAssignment {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub capacity: i64,
    pub port: i64,
    pub version: i64,
    pub status: DashboardAssignmentStatus,
}

impl DashboardAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardAssignmentStatus) -> bool {
        match self.status {
            DashboardAssignmentStatus::Draft => {
                return next == DashboardAssignmentStatus::Pending || next == DashboardAssignmentStatus::Active;
            }
            DashboardAssignmentStatus::Pending => {
                match next {
                    DashboardAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardAssignmentStatus::Active => {
                return next == DashboardAssignmentStatus::Suspended || next == DashboardAssignmentStatus::Closed;
            }
            DashboardAssignmentStatus::Suspended => {
                match next {
                    DashboardAssignmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: DashboardAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.port as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardAssignment {
        DashboardAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            capacity: self.capacity,
            port: self.port,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a dashboard assignment in status draft.
pub fn new_dashboard_assignment(code: &str, name: &str) -> DashboardAssignment {
    DashboardAssignment {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        capacity: 0,
        port: 0,
        version: 0,
        status: DashboardAssignmentStatus::Draft,
    }
}
