use super::*;

use acme_kit::text;

/// Where a health check audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckAuditStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl HealthCheckAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for HealthCheckAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A health check audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAudit {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub ttl_seconds: i64,
    pub limit: i64,
    pub attempts: i64,
    pub version: i64,
    pub visible: bool,
    pub description: String,
    pub cost: f64,
    pub weight: f64,
    pub status: HealthCheckAuditStatus,
}

impl HealthCheckAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckAuditStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckAuditStatus) -> bool {
        match self.status {
            HealthCheckAuditStatus::Proposed => {
                return next == HealthCheckAuditStatus::Withdrawn;
            }
            HealthCheckAuditStatus::Approved => {
                return next == HealthCheckAuditStatus::Rejected || next == HealthCheckAuditStatus::Withdrawn;
            }
            HealthCheckAuditStatus::Rejected => {
                match next {
                    HealthCheckAuditStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: HealthCheckAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "health check audit", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        (self.ceiling * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckAudit {
        HealthCheckAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            ttl_seconds: self.ttl_seconds,
            limit: self.limit,
            attempts: self.attempts,
            version: self.version,
            visible: self.visible,
            description: self.description.clone(),
            cost: self.cost,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a health check audit in status proposed.
pub fn new_health_check_audit(code: &str, label: &str) -> HealthCheckAudit {
    HealthCheckAudit {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        ttl_seconds: 0,
        limit: 0,
        attempts: 0,
        version: 0,
        visible: false,
        description: "".to_string(),
        cost: 0.0,
        weight: 0.0,
        status: HealthCheckAuditStatus::Proposed,
    }
}
