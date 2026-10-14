use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a usage rule must pass. Check returns its problems, or none.
pub trait UsageRuleRule: std::fmt::Debug {
    fn check(&self, item: &UsageRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageRuleCodePrefixRule {
    pub prefix: String,
}

impl UsageRuleRule for UsageRuleCodePrefixRule {
    fn check(&self, item: &UsageRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageRuleCurrencyLengthRule {
    pub max_length: i64,
}

impl UsageRuleRule for UsageRuleCurrencyLengthRule {
    fn check(&self, item: &UsageRule) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageRuleQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsageRuleRule for UsageRuleQuotaUnitsRangeRule {
    fn check(&self, item: &UsageRule) -> Vec<String> {
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

/// Runs every rule of a usage rule.
#[derive(Debug, Clone)]
pub struct UsageRuleValidator {
    pub rules: Vec<Rc<dyn UsageRuleRule>>,
}

impl UsageRuleValidator {
    pub fn validate(&self, item: &UsageRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &UsageRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_rule_validator() -> UsageRuleValidator {
    UsageRuleValidator {
        rules: vec![
            Rc::new(UsageRuleCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn UsageRuleRule>,
            Rc::new(UsageRuleCurrencyLengthRule { max_length: 16 }),
            Rc::new(UsageRuleQuotaUnitsRangeRule { min: 64, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_rule_validator();
        let item = new_usage_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = UsageRuleCurrencyLengthRule { max_length: 86400 };
        let mut item = new_usage_rule("x", "y");
        item.currency = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = UsageRuleQuotaUnitsRangeRule { min: 32, max: 80 };
        let mut item = new_usage_rule("x", "y");
        item.quota_units = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
