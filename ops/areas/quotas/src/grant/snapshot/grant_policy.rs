use super::*;

use acme_kit::text;

/// Where a grant policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantPolicyStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GrantPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GrantPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant policy as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantPolicy {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub cluster: String,
    pub discount: f64,
    pub budget: f64,
    pub retries: i64,
    pub status: GrantPolicyStatus,
}

impl GrantPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantPolicyStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantPolicyStatus) -> bool {
        match self.status {
            GrantPolicyStatus::Created => {
                match next {
                    GrantPolicyStatus::Validated | GrantPolicyStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantPolicyStatus::Validated => {
                return next == GrantPolicyStatus::Published || next == GrantPolicyStatus::Retired;
            }
            GrantPolicyStatus::Published => {
                return next == GrantPolicyStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.discount + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantPolicy {
        GrantPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            cluster: self.cluster.clone(),
            discount: self.discount,
            budget: self.budget,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a grant policy in status created.
pub fn new_grant_policy(code: &str, name: &str) -> GrantPolicy {
    GrantPolicy {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        cluster: "".to_string(),
        discount: 0.0,
        budget: 0.0,
        retries: 0,
        status: GrantPolicyStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_grant_policy("code-95", "name-898");
        assert_eq!(item.status, GrantPolicyStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_grant_policy("code-95", "name-898");
        if item.move_to(GrantPolicyStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantPolicyStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_grant_policy("code-95", "name-898");
        item.status = GrantPolicyStatus::Retired;
        assert!(
            item.move_to(GrantPolicyStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
