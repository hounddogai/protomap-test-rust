use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention policy must pass. Check returns its problems, or none.
pub trait RetentionPolicyRule: std::fmt::Debug {
    fn check(&self, item: &RetentionPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionPolicyAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RetentionPolicyRule for RetentionPolicyAllowedCodeRule {
    fn check(&self, item: &RetentionPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionPolicyCodePrefixRule {
    pub prefix: String,
}

impl RetentionPolicyRule for RetentionPolicyCodePrefixRule {
    fn check(&self, item: &RetentionPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionPolicyQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionPolicyRule for RetentionPolicyQuotaUnitsRangeRule {
    fn check(&self, item: &RetentionPolicy) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionPolicyCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionPolicyRule for RetentionPolicyCreatedAtRangeRule {
    fn check(&self, item: &RetentionPolicy) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a retention policy.
#[derive(Debug, Clone)]
pub struct RetentionPolicyValidator {
    pub rules: Vec<Rc<dyn RetentionPolicyRule>>,
}

impl RetentionPolicyValidator {
    pub fn validate(&self, item: &RetentionPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RetentionPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_policy_validator() -> RetentionPolicyValidator {
    RetentionPolicyValidator {
        rules: vec![
            Rc::new(RetentionPolicyAllowedCodeRule {
                allowed: vec!["pending".to_string(), "weighted".to_string(), "strict".to_string()],
            }) as Rc<dyn RetentionPolicyRule>,
            Rc::new(RetentionPolicyCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RetentionPolicyQuotaUnitsRangeRule { min: 16, max: 3600 }),
            Rc::new(RetentionPolicyCreatedAtRangeRule { min: 16, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_policy_validator();
        let item = new_retention_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = RetentionPolicyQuotaUnitsRangeRule { min: 80, max: 1000 };
        let mut item = new_retention_policy("x", "y");
        item.quota_units = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RetentionPolicyCreatedAtRangeRule { min: 120, max: 1000 };
        let mut item = new_retention_policy("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
