use super::*;

use acme_kit::text;

/// Where a binding quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingQuotaStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BindingQuotaStatus {
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

impl std::fmt::Display for BindingQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a binding quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingQuota {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub namespace: String,
    pub issuer: String,
    pub cost: f64,
    pub archived: bool,
    pub ttl_seconds: i64,
    pub country: String,
    pub name: String,
    pub amount: f64,
    pub status: BindingQuotaStatus,
}

impl BindingQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingQuotaStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingQuotaStatus) -> bool {
        match self.status {
            BindingQuotaStatus::Idle => {
                match next {
                    BindingQuotaStatus::Warming | BindingQuotaStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingQuotaStatus::Warming => {
                return next == BindingQuotaStatus::Draining;
            }
            BindingQuotaStatus::Ready => {
                return next == BindingQuotaStatus::Draining || next == BindingQuotaStatus::Stopped;
            }
            BindingQuotaStatus::Draining => {
                return next == BindingQuotaStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.cost - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingQuota {
        BindingQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            namespace: self.namespace.clone(),
            issuer: self.issuer.clone(),
            cost: self.cost,
            archived: self.archived,
            ttl_seconds: self.ttl_seconds,
            country: self.country.clone(),
            name: self.name.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a binding quota in status idle.
pub fn new_binding_quota(code: &str, label: &str) -> BindingQuota {
    BindingQuota {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        namespace: "".to_string(),
        issuer: "".to_string(),
        cost: 0.0,
        archived: false,
        ttl_seconds: 0,
        country: "".to_string(),
        name: "".to_string(),
        amount: 0.0,
        status: BindingQuotaStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_binding_quota("code-971", "label-773");
        assert_eq!(item.status, BindingQuotaStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_binding_quota("code-971", "label-773");
        if item.move_to(BindingQuotaStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingQuotaStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_binding_quota("code-971", "label-773");
        item.status = BindingQuotaStatus::Stopped;
        assert!(item.move_to(BindingQuotaStatus::Idle).is_err(), "expected move to fail");
    }
}
