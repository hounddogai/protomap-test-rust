use super::*;

use acme_kit::text;

/// Where a strict tenant allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictTenantAllocationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictTenantAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictTenantAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict tenant allocation as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictTenantAllocation {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub unit: String,
    pub segment: String,
    pub threshold: f64,
    pub minimum: f64,
    pub budget: f64,
    pub version: i64,
    pub ends_at: i64,
    pub contact_email: String,
    pub status: StrictTenantAllocationStatus,
}

impl StrictTenantAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictTenantAllocationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictTenantAllocationStatus) -> bool {
        match self.status {
            StrictTenantAllocationStatus::Created => {
                match next {
                    StrictTenantAllocationStatus::Validated | StrictTenantAllocationStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictTenantAllocationStatus::Validated => {
                return next == StrictTenantAllocationStatus::Published
                    || next == StrictTenantAllocationStatus::Retired;
            }
            StrictTenantAllocationStatus::Published => {
                match next {
                    StrictTenantAllocationStatus::Retired => {
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

    pub fn move_to(&mut self, next: StrictTenantAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict tenant allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.budget * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictTenantAllocation {
        StrictTenantAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            unit: self.unit.clone(),
            segment: self.segment.clone(),
            threshold: self.threshold,
            minimum: self.minimum,
            budget: self.budget,
            version: self.version,
            ends_at: self.ends_at,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a strict tenant allocation in status created.
pub fn new_strict_tenant_allocation(code: &str, name: &str) -> StrictTenantAllocation {
    StrictTenantAllocation {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        unit: "".to_string(),
        segment: "".to_string(),
        threshold: 0.0,
        minimum: 0.0,
        budget: 0.0,
        version: 0,
        ends_at: 0,
        contact_email: "".to_string(),
        status: StrictTenantAllocationStatus::Created,
    }
}
