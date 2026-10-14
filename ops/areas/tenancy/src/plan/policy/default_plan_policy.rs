use super::*;

use acme_kit::text;

/// Where a default plan policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultPlanPolicyStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultPlanPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultPlanPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default plan policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanPolicy {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub fingerprint: String,
    pub region: String,
    pub timezone: String,
    pub retries: i64,
    pub contact_email: String,
    pub locale: String,
    pub tags: Vec<String>,
    pub status: DefaultPlanPolicyStatus,
}

impl DefaultPlanPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultPlanPolicyStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultPlanPolicyStatus) -> bool {
        match self.status {
            DefaultPlanPolicyStatus::Proposed => {
                match next {
                    DefaultPlanPolicyStatus::Approved | DefaultPlanPolicyStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultPlanPolicyStatus::Approved => {
                match next {
                    DefaultPlanPolicyStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultPlanPolicyStatus::Rejected => {
                return next == DefaultPlanPolicyStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultPlanPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default plan policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default plan policy", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.retries as f64
    }

    pub fn copy(&self) -> DefaultPlanPolicy {
        DefaultPlanPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            fingerprint: self.fingerprint.clone(),
            region: self.region.clone(),
            timezone: self.timezone.clone(),
            retries: self.retries,
            contact_email: self.contact_email.clone(),
            locale: self.locale.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a default plan policy in status proposed.
pub fn new_default_plan_policy(code: &str, name: &str) -> DefaultPlanPolicy {
    DefaultPlanPolicy {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        fingerprint: "".to_string(),
        region: "".to_string(),
        timezone: "".to_string(),
        retries: 0,
        contact_email: "".to_string(),
        locale: "".to_string(),
        tags: Vec::new(),
        status: DefaultPlanPolicyStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_default_plan_policy("code-601", "name-653");
        assert_eq!(item.status, DefaultPlanPolicyStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_default_plan_policy("code-601", "name-653");
        if item.move_to(DefaultPlanPolicyStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultPlanPolicyStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_default_plan_policy("code-601", "name-653");
        item.status = DefaultPlanPolicyStatus::Withdrawn;
        assert!(
            item.move_to(DefaultPlanPolicyStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
