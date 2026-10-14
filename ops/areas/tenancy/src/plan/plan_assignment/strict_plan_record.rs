use super::*;

use acme_kit::text;

/// Where a strict plan record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictPlanRecordStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictPlanRecordStatus {
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

impl std::fmt::Display for StrictPlanRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict plan record as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPlanRecord {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub description: String,
    pub tenant_id: String,
    pub fingerprint: String,
    pub principal: String,
    pub priority: i64,
    pub limit: i64,
    pub country: String,
    pub status: StrictPlanRecordStatus,
}

impl StrictPlanRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictPlanRecordStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictPlanRecordStatus) -> bool {
        match self.status {
            StrictPlanRecordStatus::Draft => {
                return next == StrictPlanRecordStatus::Suspended || next == StrictPlanRecordStatus::Closed;
            }
            StrictPlanRecordStatus::Pending => {
                match next {
                    StrictPlanRecordStatus::Active | StrictPlanRecordStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPlanRecordStatus::Active => {
                match next {
                    StrictPlanRecordStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPlanRecordStatus::Suspended => {
                match next {
                    StrictPlanRecordStatus::Closed => {
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

    pub fn move_to(&mut self, next: StrictPlanRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict plan record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict plan record", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.priority as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictPlanRecord {
        StrictPlanRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            description: self.description.clone(),
            tenant_id: self.tenant_id.clone(),
            fingerprint: self.fingerprint.clone(),
            principal: self.principal.clone(),
            priority: self.priority,
            limit: self.limit,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a strict plan record in status draft.
pub fn new_strict_plan_record(code: &str, name: &str) -> StrictPlanRecord {
    StrictPlanRecord {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        description: "".to_string(),
        tenant_id: "".to_string(),
        fingerprint: "".to_string(),
        principal: "".to_string(),
        priority: 0,
        limit: 0,
        country: "".to_string(),
        status: StrictPlanRecordStatus::Draft,
    }
}
