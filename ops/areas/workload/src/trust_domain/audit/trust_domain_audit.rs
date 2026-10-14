use super::*;

use acme_kit::text;

/// Where a trust domain audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainAuditStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TrustDomainAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for TrustDomainAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAudit {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub approved: bool,
    pub discount: f64,
    pub summary: String,
    pub floor_value: f64,
    pub capacity: i64,
    pub country: String,
    pub cluster: String,
    pub visible: bool,
    pub status: TrustDomainAuditStatus,
}

impl TrustDomainAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainAuditStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainAuditStatus) -> bool {
        match self.status {
            TrustDomainAuditStatus::Idle => {
                match next {
                    TrustDomainAuditStatus::Warming | TrustDomainAuditStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainAuditStatus::Warming => {
                return next == TrustDomainAuditStatus::Stopped;
            }
            TrustDomainAuditStatus::Ready => {
                return next == TrustDomainAuditStatus::Stopped;
            }
            TrustDomainAuditStatus::Draining => {
                return next == TrustDomainAuditStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "trust domain audit", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.floor_value * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainAudit {
        TrustDomainAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            approved: self.approved,
            discount: self.discount,
            summary: self.summary.clone(),
            floor_value: self.floor_value,
            capacity: self.capacity,
            country: self.country.clone(),
            cluster: self.cluster.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a trust domain audit in status idle.
pub fn new_trust_domain_audit(code: &str, title: &str) -> TrustDomainAudit {
    TrustDomainAudit {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        approved: false,
        discount: 0.0,
        summary: "".to_string(),
        floor_value: 0.0,
        capacity: 0,
        country: "".to_string(),
        cluster: "".to_string(),
        visible: false,
        status: TrustDomainAuditStatus::Idle,
    }
}
