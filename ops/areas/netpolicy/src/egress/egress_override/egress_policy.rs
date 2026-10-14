use super::*;

use acme_kit::text;

/// Where a egress policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressPolicyStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl EgressPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for EgressPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One egress policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressPolicy {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub issuer: String,
    pub currency: String,
    pub retries: i64,
    pub description: String,
    pub summary: String,
    pub title: String,
    pub status: EgressPolicyStatus,
}

impl EgressPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressPolicyStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressPolicyStatus) -> bool {
        match self.status {
            EgressPolicyStatus::Created => {
                match next {
                    EgressPolicyStatus::Published | EgressPolicyStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressPolicyStatus::Validated => {
                return next == EgressPolicyStatus::Published;
            }
            EgressPolicyStatus::Published => {
                match next {
                    EgressPolicyStatus::Retired => {
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

    pub fn move_to(&mut self, next: EgressPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress policy {} cannot move from {} to {}",
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
        self.retries as f64
    }

    pub fn copy(&self) -> EgressPolicy {
        EgressPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            issuer: self.issuer.clone(),
            currency: self.currency.clone(),
            retries: self.retries,
            description: self.description.clone(),
            summary: self.summary.clone(),
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a egress policy in status created.
pub fn new_egress_policy(code: &str, label: &str) -> EgressPolicy {
    EgressPolicy {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        issuer: "".to_string(),
        currency: "".to_string(),
        retries: 0,
        description: "".to_string(),
        summary: "".to_string(),
        title: "".to_string(),
        status: EgressPolicyStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_egress_policy("code-832", "label-194");
        assert_eq!(item.status, EgressPolicyStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_egress_policy("code-832", "label-194");
        if item.move_to(EgressPolicyStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressPolicyStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_egress_policy("code-832", "label-194");
        item.status = EgressPolicyStatus::Retired;
        assert!(
            item.move_to(EgressPolicyStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
