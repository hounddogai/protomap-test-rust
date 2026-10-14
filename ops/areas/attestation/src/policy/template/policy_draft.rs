use super::*;

use acme_kit::text;

/// Where a policy draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PolicyDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PolicyDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A policy draft as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyDraft {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub issuer: String,
    pub ceiling: f64,
    pub timezone: String,
    pub key_id: String,
    pub locale: String,
    pub maximum: f64,
    pub amount: f64,
    pub status: PolicyDraftStatus,
}

impl PolicyDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyDraftStatus) -> bool {
        match self.status {
            PolicyDraftStatus::Created => {
                match next {
                    PolicyDraftStatus::Validated | PolicyDraftStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyDraftStatus::Validated => {
                match next {
                    PolicyDraftStatus::Published | PolicyDraftStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyDraftStatus::Published => {
                match next {
                    PolicyDraftStatus::Retired => {
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

    pub fn move_to(&mut self, next: PolicyDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.maximum - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyDraft {
        PolicyDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            issuer: self.issuer.clone(),
            ceiling: self.ceiling,
            timezone: self.timezone.clone(),
            key_id: self.key_id.clone(),
            locale: self.locale.clone(),
            maximum: self.maximum,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a policy draft in status created.
pub fn new_policy_draft(code: &str, name: &str) -> PolicyDraft {
    PolicyDraft {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        issuer: "".to_string(),
        ceiling: 0.0,
        timezone: "".to_string(),
        key_id: "".to_string(),
        locale: "".to_string(),
        maximum: 0.0,
        amount: 0.0,
        status: PolicyDraftStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_policy_draft("code-530", "name-355");
        assert_eq!(item.status, PolicyDraftStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_policy_draft("code-530", "name-355");
        if item.move_to(PolicyDraftStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyDraftStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_policy_draft("code-530", "name-355");
        item.status = PolicyDraftStatus::Retired;
        assert!(
            item.move_to(PolicyDraftStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
