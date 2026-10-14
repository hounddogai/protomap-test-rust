use super::*;

use acme_kit::text;

/// Where a primary peering audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryPeeringAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryPeeringAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for PrimaryPeeringAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary peering audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringAudit {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub audience: String,
    pub external_ref: String,
    pub batch_size: i64,
    pub window_days: i64,
    pub port: i64,
    pub channel: String,
    pub tenant_id: String,
    pub status: PrimaryPeeringAuditStatus,
}

impl PrimaryPeeringAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryPeeringAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryPeeringAuditStatus) -> bool {
        match self.status {
            PrimaryPeeringAuditStatus::Open => {
                match next {
                    PrimaryPeeringAuditStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryPeeringAuditStatus::InReview => {
                return next == PrimaryPeeringAuditStatus::Reopened;
            }
            PrimaryPeeringAuditStatus::Resolved => {
                return next == PrimaryPeeringAuditStatus::Reopened;
            }
            PrimaryPeeringAuditStatus::Reopened => {
                return next == PrimaryPeeringAuditStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryPeeringAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary peering audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary peering audit", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.port as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryPeeringAudit {
        PrimaryPeeringAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            audience: self.audience.clone(),
            external_ref: self.external_ref.clone(),
            batch_size: self.batch_size,
            window_days: self.window_days,
            port: self.port,
            channel: self.channel.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a primary peering audit in status open.
pub fn new_primary_peering_audit(code: &str, name: &str) -> PrimaryPeeringAudit {
    PrimaryPeeringAudit {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        audience: "".to_string(),
        external_ref: "".to_string(),
        batch_size: 0,
        window_days: 0,
        port: 0,
        channel: "".to_string(),
        tenant_id: "".to_string(),
        status: PrimaryPeeringAuditStatus::Open,
    }
}
