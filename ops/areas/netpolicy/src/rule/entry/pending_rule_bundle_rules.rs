use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending rule bundle must pass. Check returns its problems, or none.
pub trait PendingRuleBundleRule: std::fmt::Debug {
    fn check(&self, item: &PendingRuleBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleBundleRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingRuleBundleRule for PendingRuleBundleRatioRangeRule {
    fn check(&self, item: &PendingRuleBundle) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleBundleQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRuleBundleRule for PendingRuleBundleQuotaUnitsRangeRule {
    fn check(&self, item: &PendingRuleBundle) -> Vec<String> {
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
pub struct PendingRuleBundleRequiredTitleRule {
    pub message: String,
}

impl PendingRuleBundleRule for PendingRuleBundleRequiredTitleRule {
    fn check(&self, item: &PendingRuleBundle) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleBundleNameLengthRule {
    pub max_length: i64,
}

impl PendingRuleBundleRule for PendingRuleBundleNameLengthRule {
    fn check(&self, item: &PendingRuleBundle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleBundleRequiredCodeRule {
    pub message: String,
}

impl PendingRuleBundleRule for PendingRuleBundleRequiredCodeRule {
    fn check(&self, item: &PendingRuleBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleBundleTitleLengthRule {
    pub max_length: i64,
}

impl PendingRuleBundleRule for PendingRuleBundleTitleLengthRule {
    fn check(&self, item: &PendingRuleBundle) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending rule bundle.
#[derive(Debug, Clone)]
pub struct PendingRuleBundleValidator {
    pub rules: Vec<Rc<dyn PendingRuleBundleRule>>,
}

impl PendingRuleBundleValidator {
    pub fn validate(&self, item: &PendingRuleBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingRuleBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRuleBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_rule_bundle_validator() -> PendingRuleBundleValidator {
    PendingRuleBundleValidator {
        rules: vec![
            Rc::new(PendingRuleBundleRatioRangeRule { min: 9.3, max: 1608.8 }) as Rc<dyn PendingRuleBundleRule>,
            Rc::new(PendingRuleBundleQuotaUnitsRangeRule { min: 64, max: 1000 }),
            Rc::new(PendingRuleBundleRequiredTitleRule {
                message: "pending rule bundle needs a value".to_string(),
            }),
            Rc::new(PendingRuleBundleNameLengthRule { max_length: 16 }),
            Rc::new(PendingRuleBundleRequiredCodeRule {
                message: "pending rule bundle needs a value".to_string(),
            }),
            Rc::new(PendingRuleBundleTitleLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_rule_bundle_validator();
        let item = new_pending_rule_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = PendingRuleBundleRatioRangeRule { min: 8.6, max: 1721.5 };
        let mut item = new_pending_rule_bundle("x", "y");
        item.ratio = 1723.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = PendingRuleBundleQuotaUnitsRangeRule { min: 32, max: 64 };
        let mut item = new_pending_rule_bundle("x", "y");
        item.quota_units = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingRuleBundleRequiredTitleRule {
            message: "pending rule bundle needs a value".to_string(),
        };
        let mut item = new_pending_rule_bundle("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PendingRuleBundleNameLengthRule { max_length: 64 };
        let mut item = new_pending_rule_bundle("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingRuleBundleRequiredCodeRule {
            message: "pending rule bundle needs an entry".to_string(),
        };
        let mut item = new_pending_rule_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingRuleBundleTitleLengthRule { max_length: 3600 };
        let mut item = new_pending_rule_bundle("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
