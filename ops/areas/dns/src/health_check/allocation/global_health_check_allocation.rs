use super::*;

use acme_kit::text;

/// Where a global health check allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalHealthCheckAllocationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalHealthCheckAllocationStatus {
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

impl std::fmt::Display for GlobalHealthCheckAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global health check allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalHealthCheckAllocation {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub contact_email: String,
    pub port: i64,
    pub capacity: i64,
    pub updated_at: i64,
    pub archived: bool,
    pub status: GlobalHealthCheckAllocationStatus,
}

impl GlobalHealthCheckAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalHealthCheckAllocationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalHealthCheckAllocationStatus) -> bool {
        match self.status {
            GlobalHealthCheckAllocationStatus::Draft => {
                match next {
                    GlobalHealthCheckAllocationStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalHealthCheckAllocationStatus::Pending => {
                match next {
                    GlobalHealthCheckAllocationStatus::Active | GlobalHealthCheckAllocationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalHealthCheckAllocationStatus::Active => {
                return next == GlobalHealthCheckAllocationStatus::Suspended
                    || next == GlobalHealthCheckAllocationStatus::Closed;
            }
            GlobalHealthCheckAllocationStatus::Suspended => {
                match next {
                    GlobalHealthCheckAllocationStatus::Closed => {
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

    pub fn move_to(&mut self, next: GlobalHealthCheckAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global health check allocation {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "global health check allocation", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.capacity as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalHealthCheckAllocation {
        GlobalHealthCheckAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            contact_email: self.contact_email.clone(),
            port: self.port,
            capacity: self.capacity,
            updated_at: self.updated_at,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global health check allocation in status draft.
pub fn new_global_health_check_allocation(code: &str, title: &str) -> GlobalHealthCheckAllocation {
    GlobalHealthCheckAllocation {
        code: code.to_string(),
        title: title.to_string(),
        phone: "".to_string(),
        contact_email: "".to_string(),
        port: 0,
        capacity: 0,
        updated_at: 0,
        archived: false,
        status: GlobalHealthCheckAllocationStatus::Draft,
    }
}
