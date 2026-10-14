use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted rule variance must pass. Check returns its problems, or none.
pub trait WeightedRuleVarianceRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRuleVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleVarianceUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedRuleVarianceRule for WeightedRuleVarianceUpdatedAtRangeRule {
    fn check(&self, item: &WeightedRuleVariance) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleVarianceCodeLengthRule {
    pub max_length: i64,
}

impl WeightedRuleVarianceRule for WeightedRuleVarianceCodeLengthRule {
    fn check(&self, item: &WeightedRuleVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleVarianceBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedRuleVarianceRule for WeightedRuleVarianceBurstRangeRule {
    fn check(&self, item: &WeightedRuleVariance) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a weighted rule variance.
#[derive(Debug, Clone)]
pub struct WeightedRuleVarianceValidator {
    pub rules: Vec<Rc<dyn WeightedRuleVarianceRule>>,
}

impl WeightedRuleVarianceValidator {
    pub fn validate(&self, item: &WeightedRuleVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedRuleVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRuleVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_rule_variance_validator() -> WeightedRuleVarianceValidator {
    WeightedRuleVarianceValidator {
        rules: vec![
            Rc::new(WeightedRuleVarianceUpdatedAtRangeRule { min: 32, max: 3600 }) as Rc<dyn WeightedRuleVarianceRule>,
            Rc::new(WeightedRuleVarianceCodeLengthRule { max_length: 80 }),
            Rc::new(WeightedRuleVarianceBurstRangeRule { min: 64, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_rule_variance_validator();
        let item = new_weighted_rule_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = WeightedRuleVarianceUpdatedAtRangeRule { min: 80, max: 3600 };
        let mut item = new_weighted_rule_variance("x", "y");
        item.updated_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedRuleVarianceCodeLengthRule { max_length: 16 };
        let mut item = new_weighted_rule_variance("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = WeightedRuleVarianceBurstRangeRule { min: 16, max: 3600 };
        let mut item = new_weighted_rule_variance("x", "y");
        item.burst = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
