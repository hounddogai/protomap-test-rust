use super::*;

use acme_kit::text;

/// Where a strict identity policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictIdentityPolicyStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictIdentityPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictIdentityPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict identity policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictIdentityPolicy {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub budget: f64,
    pub key_id: String,
    pub minimum: f64,
    pub audience: String,
    pub status: StrictIdentityPolicyStatus,
}

impl StrictIdentityPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictIdentityPolicyStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictIdentityPolicyStatus) -> bool {
        match self.status {
            StrictIdentityPolicyStatus::Created => {
                match next {
                    StrictIdentityPolicyStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictIdentityPolicyStatus::Validated => {
                match next {
                    StrictIdentityPolicyStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictIdentityPolicyStatus::Published => {
                return next == StrictIdentityPolicyStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictIdentityPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict identity policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict identity policy", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.floor_value * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictIdentityPolicy {
        StrictIdentityPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            budget: self.budget,
            key_id: self.key_id.clone(),
            minimum: self.minimum,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict identity policy in status created.
pub fn new_strict_identity_policy(code: &str, title: &str) -> StrictIdentityPolicy {
    StrictIdentityPolicy {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        budget: 0.0,
        key_id: "".to_string(),
        minimum: 0.0,
        audience: "".to_string(),
        status: StrictIdentityPolicyStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_strict_identity_policy("code-687", "title-873");
        assert_eq!(item.status, StrictIdentityPolicyStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_strict_identity_policy("code-687", "title-873");
        if item.move_to(StrictIdentityPolicyStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictIdentityPolicyStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_strict_identity_policy("code-687", "title-873");
        item.status = StrictIdentityPolicyStatus::Retired;
        assert!(
            item.move_to(StrictIdentityPolicyStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
