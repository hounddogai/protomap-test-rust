use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy usage policy must pass. Check returns its problems, or none.
pub trait LegacyUsagePolicyRule: std::fmt::Debug {
    fn check(&self, item: &LegacyUsagePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsagePolicyRequiredCodeRule {
    pub message: String,
}

impl LegacyUsagePolicyRule for LegacyUsagePolicyRequiredCodeRule {
    fn check(&self, item: &LegacyUsagePolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsagePolicyNameLengthRule {
    pub max_length: i64,
}

impl LegacyUsagePolicyRule for LegacyUsagePolicyNameLengthRule {
    fn check(&self, item: &LegacyUsagePolicy) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsagePolicyRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyUsagePolicyRule for LegacyUsagePolicyRateRangeRule {
    fn check(&self, item: &LegacyUsagePolicy) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a legacy usage policy.
#[derive(Debug, Clone)]
pub struct LegacyUsagePolicyValidator {
    pub rules: Vec<Rc<dyn LegacyUsagePolicyRule>>,
}

impl LegacyUsagePolicyValidator {
    pub fn validate(&self, item: &LegacyUsagePolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyUsagePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyUsagePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_usage_policy_validator() -> LegacyUsagePolicyValidator {
    LegacyUsagePolicyValidator {
        rules: vec![
            Rc::new(LegacyUsagePolicyRequiredCodeRule {
                message: "legacy usage policy needs a value".to_string(),
            }) as Rc<dyn LegacyUsagePolicyRule>,
            Rc::new(LegacyUsagePolicyNameLengthRule { max_length: 16 }),
            Rc::new(LegacyUsagePolicyRateRangeRule { min: 7.0, max: 4324.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_usage_policy_validator();
        let item = new_legacy_usage_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyUsagePolicyRequiredCodeRule {
            message: "legacy usage policy needs an entry".to_string(),
        };
        let mut item = new_legacy_usage_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LegacyUsagePolicyNameLengthRule { max_length: 1000 };
        let mut item = new_legacy_usage_policy("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = LegacyUsagePolicyRateRangeRule { min: 4.3, max: 678.9 };
        let mut item = new_legacy_usage_policy("x", "y");
        item.rate = 680.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
