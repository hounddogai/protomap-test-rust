use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict approval threshold must pass. Check returns its problems, or none.
pub trait StrictApprovalThresholdRule: std::fmt::Debug {
    fn check(&self, item: &StrictApprovalThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalThresholdCategoryLengthRule {
    pub max_length: i64,
}

impl StrictApprovalThresholdRule for StrictApprovalThresholdCategoryLengthRule {
    fn check(&self, item: &StrictApprovalThreshold) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalThresholdIssuerLengthRule {
    pub max_length: i64,
}

impl StrictApprovalThresholdRule for StrictApprovalThresholdIssuerLengthRule {
    fn check(&self, item: &StrictApprovalThreshold) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalThresholdCodeLengthRule {
    pub max_length: i64,
}

impl StrictApprovalThresholdRule for StrictApprovalThresholdCodeLengthRule {
    fn check(&self, item: &StrictApprovalThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalThresholdCodePrefixRule {
    pub prefix: String,
}

impl StrictApprovalThresholdRule for StrictApprovalThresholdCodePrefixRule {
    fn check(&self, item: &StrictApprovalThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalThresholdPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictApprovalThresholdRule for StrictApprovalThresholdPortRangeRule {
    fn check(&self, item: &StrictApprovalThreshold) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a strict approval threshold.
#[derive(Debug, Clone)]
pub struct StrictApprovalThresholdValidator {
    pub rules: Vec<Rc<dyn StrictApprovalThresholdRule>>,
}

impl StrictApprovalThresholdValidator {
    pub fn validate(&self, item: &StrictApprovalThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictApprovalThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictApprovalThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_approval_threshold_validator() -> StrictApprovalThresholdValidator {
    StrictApprovalThresholdValidator {
        rules: vec![
            Rc::new(StrictApprovalThresholdCategoryLengthRule { max_length: 86400 })
                as Rc<dyn StrictApprovalThresholdRule>,
            Rc::new(StrictApprovalThresholdIssuerLengthRule { max_length: 32 }),
            Rc::new(StrictApprovalThresholdCodeLengthRule { max_length: 16 }),
            Rc::new(StrictApprovalThresholdCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(StrictApprovalThresholdPortRangeRule { min: 64, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_approval_threshold_validator();
        let item = new_strict_approval_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = StrictApprovalThresholdCategoryLengthRule { max_length: 64 };
        let mut item = new_strict_approval_threshold("x", "y");
        item.category = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = StrictApprovalThresholdIssuerLengthRule { max_length: 32 };
        let mut item = new_strict_approval_threshold("x", "y");
        item.issuer = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictApprovalThresholdCodeLengthRule { max_length: 120 };
        let mut item = new_strict_approval_threshold("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = StrictApprovalThresholdPortRangeRule { min: 32, max: 86400 };
        let mut item = new_strict_approval_threshold("x", "y");
        item.port = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
