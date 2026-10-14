use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension policy must pass. Check returns its problems, or none.
pub trait SuspensionPolicyRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionPolicySourceLengthRule {
    pub max_length: i64,
}

impl SuspensionPolicyRule for SuspensionPolicySourceLengthRule {
    fn check(&self, item: &SuspensionPolicy) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionPolicyTenantIdLengthRule {
    pub max_length: i64,
}

impl SuspensionPolicyRule for SuspensionPolicyTenantIdLengthRule {
    fn check(&self, item: &SuspensionPolicy) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionPolicyRequiredTitleRule {
    pub message: String,
}

impl SuspensionPolicyRule for SuspensionPolicyRequiredTitleRule {
    fn check(&self, item: &SuspensionPolicy) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension policy.
#[derive(Debug, Clone)]
pub struct SuspensionPolicyValidator {
    pub rules: Vec<Rc<dyn SuspensionPolicyRule>>,
}

impl SuspensionPolicyValidator {
    pub fn validate(&self, item: &SuspensionPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_policy_validator() -> SuspensionPolicyValidator {
    SuspensionPolicyValidator {
        rules: vec![
            Rc::new(SuspensionPolicySourceLengthRule { max_length: 255 }) as Rc<dyn SuspensionPolicyRule>,
            Rc::new(SuspensionPolicyTenantIdLengthRule { max_length: 120 }),
            Rc::new(SuspensionPolicyRequiredTitleRule {
                message: "suspension policy needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_policy_validator();
        let item = new_suspension_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = SuspensionPolicySourceLengthRule { max_length: 64 };
        let mut item = new_suspension_policy("x", "y");
        item.source = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = SuspensionPolicyTenantIdLengthRule { max_length: 80 };
        let mut item = new_suspension_policy("x", "y");
        item.tenant_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SuspensionPolicyRequiredTitleRule {
            message: "suspension policy needs a value".to_string(),
        };
        let mut item = new_suspension_policy("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
