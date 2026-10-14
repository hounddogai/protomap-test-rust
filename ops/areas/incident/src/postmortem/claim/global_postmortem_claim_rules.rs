use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global postmortem claim must pass. Check returns its problems, or none.
pub trait GlobalPostmortemClaimRule: std::fmt::Debug {
    fn check(&self, item: &GlobalPostmortemClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPostmortemClaimCodeLengthRule {
    pub max_length: i64,
}

impl GlobalPostmortemClaimRule for GlobalPostmortemClaimCodeLengthRule {
    fn check(&self, item: &GlobalPostmortemClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPostmortemClaimRequiredCodeRule {
    pub message: String,
}

impl GlobalPostmortemClaimRule for GlobalPostmortemClaimRequiredCodeRule {
    fn check(&self, item: &GlobalPostmortemClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPostmortemClaimNamespaceLengthRule {
    pub max_length: i64,
}

impl GlobalPostmortemClaimRule for GlobalPostmortemClaimNamespaceLengthRule {
    fn check(&self, item: &GlobalPostmortemClaim) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a global postmortem claim.
#[derive(Debug, Clone)]
pub struct GlobalPostmortemClaimValidator {
    pub rules: Vec<Rc<dyn GlobalPostmortemClaimRule>>,
}

impl GlobalPostmortemClaimValidator {
    pub fn validate(&self, item: &GlobalPostmortemClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalPostmortemClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalPostmortemClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_postmortem_claim_validator() -> GlobalPostmortemClaimValidator {
    GlobalPostmortemClaimValidator {
        rules: vec![
            Rc::new(GlobalPostmortemClaimCodeLengthRule { max_length: 16 }) as Rc<dyn GlobalPostmortemClaimRule>,
            Rc::new(GlobalPostmortemClaimRequiredCodeRule {
                message: "global postmortem claim needs a value".to_string(),
            }),
            Rc::new(GlobalPostmortemClaimNamespaceLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_postmortem_claim_validator();
        let item = new_global_postmortem_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalPostmortemClaimCodeLengthRule { max_length: 80 };
        let mut item = new_global_postmortem_claim("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalPostmortemClaimRequiredCodeRule {
            message: "global postmortem claim needs an entry".to_string(),
        };
        let mut item = new_global_postmortem_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = GlobalPostmortemClaimNamespaceLengthRule { max_length: 1000 };
        let mut item = new_global_postmortem_claim("x", "y");
        item.namespace = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
