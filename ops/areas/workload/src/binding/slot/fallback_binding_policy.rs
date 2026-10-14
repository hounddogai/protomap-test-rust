use super::*;

use acme_kit::text;

/// Where a fallback binding policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackBindingPolicyStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackBindingPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackBindingPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback binding policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingPolicy {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub visible: bool,
    pub discount: f64,
    pub audience: String,
    pub revision: i64,
    pub score: f64,
    pub unit: String,
    pub status: FallbackBindingPolicyStatus,
}

impl FallbackBindingPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackBindingPolicyStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackBindingPolicyStatus) -> bool {
        match self.status {
            FallbackBindingPolicyStatus::Requested => {
                match next {
                    FallbackBindingPolicyStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackBindingPolicyStatus::Granted => {
                return next == FallbackBindingPolicyStatus::Revoked || next == FallbackBindingPolicyStatus::Expired;
            }
            FallbackBindingPolicyStatus::Revoked => {
                return next == FallbackBindingPolicyStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackBindingPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback binding policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.discount - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackBindingPolicy {
        FallbackBindingPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            visible: self.visible,
            discount: self.discount,
            audience: self.audience.clone(),
            revision: self.revision,
            score: self.score,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a fallback binding policy in status requested.
pub fn new_fallback_binding_policy(code: &str, name: &str) -> FallbackBindingPolicy {
    FallbackBindingPolicy {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        visible: false,
        discount: 0.0,
        audience: "".to_string(),
        revision: 0,
        score: 0.0,
        unit: "".to_string(),
        status: FallbackBindingPolicyStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_fallback_binding_policy("code-528", "name-941");
        assert_eq!(item.status, FallbackBindingPolicyStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_fallback_binding_policy("code-528", "name-941");
        if item.move_to(FallbackBindingPolicyStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackBindingPolicyStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_fallback_binding_policy("code-528", "name-941");
        item.status = FallbackBindingPolicyStatus::Expired;
        assert!(
            item.move_to(FallbackBindingPolicyStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
