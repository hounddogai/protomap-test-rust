use super::*;

use acme_kit::text;

/// Where a limit policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitPolicyStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LimitPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LimitPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitPolicy {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub scope: String,
    pub phone: String,
    pub description: String,
    pub floor_value: f64,
    pub port: i64,
    pub burst: i64,
    pub priority: i64,
    pub external_ref: String,
    pub status: LimitPolicyStatus,
}

impl LimitPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitPolicyStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitPolicyStatus) -> bool {
        match self.status {
            LimitPolicyStatus::Requested => {
                return next == LimitPolicyStatus::Granted || next == LimitPolicyStatus::Revoked;
            }
            LimitPolicyStatus::Granted => {
                return next == LimitPolicyStatus::Revoked || next == LimitPolicyStatus::Expired;
            }
            LimitPolicyStatus::Revoked => {
                return next == LimitPolicyStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.floor_value * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitPolicy {
        LimitPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            scope: self.scope.clone(),
            phone: self.phone.clone(),
            description: self.description.clone(),
            floor_value: self.floor_value,
            port: self.port,
            burst: self.burst,
            priority: self.priority,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a limit policy in status requested.
pub fn new_limit_policy(code: &str, label: &str) -> LimitPolicy {
    LimitPolicy {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        scope: "".to_string(),
        phone: "".to_string(),
        description: "".to_string(),
        floor_value: 0.0,
        port: 0,
        burst: 0,
        priority: 0,
        external_ref: "".to_string(),
        status: LimitPolicyStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_limit_policy("code-411", "label-282");
        assert_eq!(item.status, LimitPolicyStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_limit_policy("code-411", "label-282");
        if item.move_to(LimitPolicyStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitPolicyStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_limit_policy("code-411", "label-282");
        item.status = LimitPolicyStatus::Expired;
        assert!(
            item.move_to(LimitPolicyStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
