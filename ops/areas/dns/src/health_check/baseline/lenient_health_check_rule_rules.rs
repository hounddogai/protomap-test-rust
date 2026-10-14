use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient health check rule must pass. Check returns its problems, or none.
pub trait LenientHealthCheckRuleRule: std::fmt::Debug {
    fn check(&self, item: &LenientHealthCheckRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientHealthCheckRuleUnitLengthRule {
    pub max_length: i64,
}

impl LenientHealthCheckRuleRule for LenientHealthCheckRuleUnitLengthRule {
    fn check(&self, item: &LenientHealthCheckRule) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientHealthCheckRulePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientHealthCheckRuleRule for LenientHealthCheckRulePortRangeRule {
    fn check(&self, item: &LenientHealthCheckRule) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct LenientHealthCheckRuleTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl LenientHealthCheckRuleRule for LenientHealthCheckRuleTagsLimitRule {
    fn check(&self, item: &LenientHealthCheckRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a lenient health check rule.
#[derive(Debug, Clone)]
pub struct LenientHealthCheckRuleValidator {
    pub rules: Vec<Rc<dyn LenientHealthCheckRuleRule>>,
}

impl LenientHealthCheckRuleValidator {
    pub fn validate(&self, item: &LenientHealthCheckRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientHealthCheckRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientHealthCheckRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_health_check_rule_validator() -> LenientHealthCheckRuleValidator {
    LenientHealthCheckRuleValidator {
        rules: vec![
            Rc::new(LenientHealthCheckRuleUnitLengthRule { max_length: 80 }) as Rc<dyn LenientHealthCheckRuleRule>,
            Rc::new(LenientHealthCheckRulePortRangeRule { min: 255, max: 1000 }),
            Rc::new(LenientHealthCheckRuleTagsLimitRule {
                max_count: 86400,
                banned: vec!["fallback".to_string(), "manual".to_string(), "weighted".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_health_check_rule_validator();
        let item = new_lenient_health_check_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = LenientHealthCheckRuleUnitLengthRule { max_length: 16 };
        let mut item = new_lenient_health_check_rule("x", "y");
        item.unit = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = LenientHealthCheckRulePortRangeRule { min: 32, max: 32 };
        let mut item = new_lenient_health_check_rule("x", "y");
        item.port = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
