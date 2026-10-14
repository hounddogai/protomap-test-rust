use super::*;

use acme_kit::text;

/// Where a weighted ttl policy quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedTtlPolicyQuotaStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedTtlPolicyQuotaStatus {
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

impl std::fmt::Display for WeightedTtlPolicyQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted ttl policy quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyQuota {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub description: String,
    pub port: i64,
    pub timezone: String,
    pub principal: String,
    pub external_ref: String,
    pub status: WeightedTtlPolicyQuotaStatus,
}

impl WeightedTtlPolicyQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedTtlPolicyQuotaStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedTtlPolicyQuotaStatus) -> bool {
        match self.status {
            WeightedTtlPolicyQuotaStatus::Idle => {
                return next == WeightedTtlPolicyQuotaStatus::Warming;
            }
            WeightedTtlPolicyQuotaStatus::Warming => {
                match next {
                    WeightedTtlPolicyQuotaStatus::Ready | WeightedTtlPolicyQuotaStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTtlPolicyQuotaStatus::Ready => {
                match next {
                    WeightedTtlPolicyQuotaStatus::Draining | WeightedTtlPolicyQuotaStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTtlPolicyQuotaStatus::Draining => {
                return next == WeightedTtlPolicyQuotaStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedTtlPolicyQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted ttl policy quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.port as f64
    }

    pub fn copy(&self) -> WeightedTtlPolicyQuota {
        WeightedTtlPolicyQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            description: self.description.clone(),
            port: self.port,
            timezone: self.timezone.clone(),
            principal: self.principal.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a weighted ttl policy quota in status idle.
pub fn new_weighted_ttl_policy_quota(code: &str, label: &str) -> WeightedTtlPolicyQuota {
    WeightedTtlPolicyQuota {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        description: "".to_string(),
        port: 0,
        timezone: "".to_string(),
        principal: "".to_string(),
        external_ref: "".to_string(),
        status: WeightedTtlPolicyQuotaStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_ttl_policy_quota("code-362", "label-492");
        assert_eq!(item.status, WeightedTtlPolicyQuotaStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_weighted_ttl_policy_quota("code-362", "label-492");
        if item.move_to(WeightedTtlPolicyQuotaStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedTtlPolicyQuotaStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_ttl_policy_quota("code-362", "label-492");
        item.status = WeightedTtlPolicyQuotaStatus::Stopped;
        assert!(
            item.move_to(WeightedTtlPolicyQuotaStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
