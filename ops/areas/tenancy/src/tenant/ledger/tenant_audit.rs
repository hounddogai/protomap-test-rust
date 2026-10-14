use super::*;

use acme_kit::text;

/// Where a tenant audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantAuditStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TenantAuditStatus {
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

impl std::fmt::Display for TenantAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantAudit {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub discount: f64,
    pub pinned: bool,
    pub timezone: String,
    pub duration_minutes: i64,
    pub status: TenantAuditStatus,
}

impl TenantAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantAuditStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantAuditStatus) -> bool {
        match self.status {
            TenantAuditStatus::Idle => {
                match next {
                    TenantAuditStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantAuditStatus::Warming => {
                return next == TenantAuditStatus::Ready || next == TenantAuditStatus::Draining;
            }
            TenantAuditStatus::Ready => {
                return next == TenantAuditStatus::Stopped;
            }
            TenantAuditStatus::Draining => {
                match next {
                    TenantAuditStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TenantAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tenant audit", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.duration_minutes as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantAudit {
        TenantAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            discount: self.discount,
            pinned: self.pinned,
            timezone: self.timezone.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tenant audit in status idle.
pub fn new_tenant_audit(code: &str, title: &str) -> TenantAudit {
    TenantAudit {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        discount: 0.0,
        pinned: false,
        timezone: "".to_string(),
        duration_minutes: 0,
        status: TenantAuditStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tenant_audit("code-163", "title-853");
        assert_eq!(item.status, TenantAuditStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_tenant_audit("code-163", "title-853");
        if item.move_to(TenantAuditStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantAuditStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tenant_audit("code-163", "title-853");
        item.status = TenantAuditStatus::Stopped;
        assert!(item.move_to(TenantAuditStatus::Idle).is_err(), "expected move to fail");
    }
}
