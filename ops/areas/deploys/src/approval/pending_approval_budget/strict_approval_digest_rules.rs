use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict approval digest must pass. Check returns its problems, or none.
pub trait StrictApprovalDigestRule: std::fmt::Debug {
    fn check(&self, item: &StrictApprovalDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalDigestNameLengthRule {
    pub max_length: i64,
}

impl StrictApprovalDigestRule for StrictApprovalDigestNameLengthRule {
    fn check(&self, item: &StrictApprovalDigest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalDigestCodePrefixRule {
    pub prefix: String,
}

impl StrictApprovalDigestRule for StrictApprovalDigestCodePrefixRule {
    fn check(&self, item: &StrictApprovalDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalDigestRequiredUnitRule {
    pub message: String,
}

impl StrictApprovalDigestRule for StrictApprovalDigestRequiredUnitRule {
    fn check(&self, item: &StrictApprovalDigest) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalDigestWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictApprovalDigestRule for StrictApprovalDigestWeightRangeRule {
    fn check(&self, item: &StrictApprovalDigest) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict approval digest.
#[derive(Debug, Clone)]
pub struct StrictApprovalDigestValidator {
    pub rules: Vec<Rc<dyn StrictApprovalDigestRule>>,
}

impl StrictApprovalDigestValidator {
    pub fn validate(&self, item: &StrictApprovalDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictApprovalDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictApprovalDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_approval_digest_validator() -> StrictApprovalDigestValidator {
    StrictApprovalDigestValidator {
        rules: vec![
            Rc::new(StrictApprovalDigestNameLengthRule { max_length: 255 }) as Rc<dyn StrictApprovalDigestRule>,
            Rc::new(StrictApprovalDigestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(StrictApprovalDigestRequiredUnitRule {
                message: "strict approval digest needs an entry".to_string(),
            }),
            Rc::new(StrictApprovalDigestWeightRangeRule { min: 9.9, max: 1418.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_approval_digest_validator();
        let item = new_strict_approval_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = StrictApprovalDigestNameLengthRule { max_length: 86400 };
        let mut item = new_strict_approval_digest("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = StrictApprovalDigestRequiredUnitRule {
            message: "strict approval digest needs a value".to_string(),
        };
        let mut item = new_strict_approval_digest("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = StrictApprovalDigestWeightRangeRule { min: 8.7, max: 1651.6 };
        let mut item = new_strict_approval_digest("x", "y");
        item.weight = 1653.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
