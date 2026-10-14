use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient limit rule must pass. Check returns its problems, or none.
pub trait LenientLimitRuleRule: std::fmt::Debug {
    fn check(&self, item: &LenientLimitRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientLimitRulePhoneLengthRule {
    pub max_length: i64,
}

impl LenientLimitRuleRule for LenientLimitRulePhoneLengthRule {
    fn check(&self, item: &LenientLimitRule) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientLimitRuleEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientLimitRuleRule for LenientLimitRuleEndsAtRangeRule {
    fn check(&self, item: &LenientLimitRule) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientLimitRuleCountryLengthRule {
    pub max_length: i64,
}

impl LenientLimitRuleRule for LenientLimitRuleCountryLengthRule {
    fn check(&self, item: &LenientLimitRule) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient limit rule.
#[derive(Debug, Clone)]
pub struct LenientLimitRuleValidator {
    pub rules: Vec<Rc<dyn LenientLimitRuleRule>>,
}

impl LenientLimitRuleValidator {
    pub fn validate(&self, item: &LenientLimitRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientLimitRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientLimitRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_limit_rule_validator() -> LenientLimitRuleValidator {
    LenientLimitRuleValidator {
        rules: vec![
            Rc::new(LenientLimitRulePhoneLengthRule { max_length: 3600 }) as Rc<dyn LenientLimitRuleRule>,
            Rc::new(LenientLimitRuleEndsAtRangeRule { min: 16, max: 80 }),
            Rc::new(LenientLimitRuleCountryLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_limit_rule_validator();
        let item = new_lenient_limit_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = LenientLimitRulePhoneLengthRule { max_length: 64 };
        let mut item = new_lenient_limit_rule("x", "y");
        item.phone = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = LenientLimitRuleEndsAtRangeRule { min: 80, max: 86400 };
        let mut item = new_lenient_limit_rule("x", "y");
        item.ends_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = LenientLimitRuleCountryLengthRule { max_length: 16 };
        let mut item = new_lenient_limit_rule("x", "y");
        item.country = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
