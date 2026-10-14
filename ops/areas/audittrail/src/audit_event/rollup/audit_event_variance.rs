use super::*;

use acme_kit::text;

/// Where a audit event variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventVarianceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AuditEventVarianceStatus {
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

impl std::fmt::Display for AuditEventVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event variance as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventVariance {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub discount: f64,
    pub locale: String,
    pub rotation_days: i64,
    pub namespace: String,
    pub retries: i64,
    pub owner_id: String,
    pub grace_days: i64,
    pub status: AuditEventVarianceStatus,
}

impl AuditEventVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventVarianceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventVarianceStatus) -> bool {
        match self.status {
            AuditEventVarianceStatus::Idle => {
                return next == AuditEventVarianceStatus::Stopped;
            }
            AuditEventVarianceStatus::Warming => {
                match next {
                    AuditEventVarianceStatus::Draining | AuditEventVarianceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventVarianceStatus::Ready => {
                return next == AuditEventVarianceStatus::Stopped;
            }
            AuditEventVarianceStatus::Draining => {
                match next {
                    AuditEventVarianceStatus::Stopped => {
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

    pub fn move_to(&mut self, next: AuditEventVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.weight + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventVariance {
        AuditEventVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            discount: self.discount,
            locale: self.locale.clone(),
            rotation_days: self.rotation_days,
            namespace: self.namespace.clone(),
            retries: self.retries,
            owner_id: self.owner_id.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a audit event variance in status idle.
pub fn new_audit_event_variance(code: &str, name: &str) -> AuditEventVariance {
    AuditEventVariance {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        discount: 0.0,
        locale: "".to_string(),
        rotation_days: 0,
        namespace: "".to_string(),
        retries: 0,
        owner_id: "".to_string(),
        grace_days: 0,
        status: AuditEventVarianceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_audit_event_variance("code-729", "name-329");
        assert_eq!(item.status, AuditEventVarianceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_audit_event_variance("code-729", "name-329");
        if item.move_to(AuditEventVarianceStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventVarianceStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_audit_event_variance("code-729", "name-329");
        item.status = AuditEventVarianceStatus::Stopped;
        assert!(
            item.move_to(AuditEventVarianceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
