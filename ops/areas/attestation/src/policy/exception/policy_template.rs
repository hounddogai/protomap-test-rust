use super::*;

use acme_kit::text;

/// Where a policy template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyTemplateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PolicyTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PolicyTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A policy template as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyTemplate {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub amount: f64,
    pub category: String,
    pub country: String,
    pub notes: String,
    pub weight: f64,
    pub capacity: i64,
    pub description: String,
    pub status: PolicyTemplateStatus,
}

impl PolicyTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyTemplateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyTemplateStatus) -> bool {
        match self.status {
            PolicyTemplateStatus::Requested => {
                match next {
                    PolicyTemplateStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyTemplateStatus::Granted => {
                return next == PolicyTemplateStatus::Expired;
            }
            PolicyTemplateStatus::Revoked => {
                match next {
                    PolicyTemplateStatus::Expired => {
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

    pub fn move_to(&mut self, next: PolicyTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "policy template", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.weight + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyTemplate {
        PolicyTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            amount: self.amount,
            category: self.category.clone(),
            country: self.country.clone(),
            notes: self.notes.clone(),
            weight: self.weight,
            capacity: self.capacity,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a policy template in status requested.
pub fn new_policy_template(code: &str, label: &str) -> PolicyTemplate {
    PolicyTemplate {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        amount: 0.0,
        category: "".to_string(),
        country: "".to_string(),
        notes: "".to_string(),
        weight: 0.0,
        capacity: 0,
        description: "".to_string(),
        status: PolicyTemplateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_policy_template("code-856", "label-126");
        assert_eq!(item.status, PolicyTemplateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_policy_template("code-856", "label-126");
        if item.move_to(PolicyTemplateStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyTemplateStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_policy_template("code-856", "label-126");
        item.status = PolicyTemplateStatus::Expired;
        assert!(
            item.move_to(PolicyTemplateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
