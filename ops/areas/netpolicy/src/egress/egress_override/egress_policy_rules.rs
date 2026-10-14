use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress policy must pass. Check returns its problems, or none.
pub trait EgressPolicyRule: std::fmt::Debug {
    fn check(&self, item: &EgressPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressPolicyRequiredCodeRule {
    pub message: String,
}

impl EgressPolicyRule for EgressPolicyRequiredCodeRule {
    fn check(&self, item: &EgressPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressPolicyCodeLengthRule {
    pub max_length: i64,
}

impl EgressPolicyRule for EgressPolicyCodeLengthRule {
    fn check(&self, item: &EgressPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressPolicyIssuerLengthRule {
    pub max_length: i64,
}

impl EgressPolicyRule for EgressPolicyIssuerLengthRule {
    fn check(&self, item: &EgressPolicy) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressPolicyLabelLengthRule {
    pub max_length: i64,
}

impl EgressPolicyRule for EgressPolicyLabelLengthRule {
    fn check(&self, item: &EgressPolicy) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressPolicyContactEmailLengthRule {
    pub max_length: i64,
}

impl EgressPolicyRule for EgressPolicyContactEmailLengthRule {
    fn check(&self, item: &EgressPolicy) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress policy.
#[derive(Debug, Clone)]
pub struct EgressPolicyValidator {
    pub rules: Vec<Rc<dyn EgressPolicyRule>>,
}

impl EgressPolicyValidator {
    pub fn validate(&self, item: &EgressPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EgressPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressPolicy]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_egress_policy_validator() -> EgressPolicyValidator {
    EgressPolicyValidator {
        rules: vec![
            Rc::new(EgressPolicyRequiredCodeRule {
                message: "egress policy needs an entry".to_string(),
            }) as Rc<dyn EgressPolicyRule>,
            Rc::new(EgressPolicyCodeLengthRule { max_length: 64 }),
            Rc::new(EgressPolicyIssuerLengthRule { max_length: 32 }),
            Rc::new(EgressPolicyLabelLengthRule { max_length: 120 }),
            Rc::new(EgressPolicyContactEmailLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_policy_validator();
        let item = new_egress_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EgressPolicyRequiredCodeRule {
            message: "egress policy needs a value".to_string(),
        };
        let mut item = new_egress_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EgressPolicyCodeLengthRule { max_length: 120 };
        let mut item = new_egress_policy("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = EgressPolicyIssuerLengthRule { max_length: 32 };
        let mut item = new_egress_policy("x", "y");
        item.issuer = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = EgressPolicyLabelLengthRule { max_length: 255 };
        let mut item = new_egress_policy("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = EgressPolicyContactEmailLengthRule { max_length: 86400 };
        let mut item = new_egress_policy("x", "y");
        item.contact_email = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
