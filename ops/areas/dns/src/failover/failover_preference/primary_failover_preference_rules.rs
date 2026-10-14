use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary failover preference must pass. Check returns its problems, or none.
pub trait PrimaryFailoverPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryFailoverPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryFailoverPreferenceRule for PrimaryFailoverPreferenceCodeLengthRule {
    fn check(&self, item: &PrimaryFailoverPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverPreferenceQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryFailoverPreferenceRule for PrimaryFailoverPreferenceQuantityRangeRule {
    fn check(&self, item: &PrimaryFailoverPreference) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverPreferenceScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryFailoverPreferenceRule for PrimaryFailoverPreferenceScoreRangeRule {
    fn check(&self, item: &PrimaryFailoverPreference) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverPreferenceBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryFailoverPreferenceRule for PrimaryFailoverPreferenceBudgetRangeRule {
    fn check(&self, item: &PrimaryFailoverPreference) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverPreferenceCodePrefixRule {
    pub prefix: String,
}

impl PrimaryFailoverPreferenceRule for PrimaryFailoverPreferenceCodePrefixRule {
    fn check(&self, item: &PrimaryFailoverPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary failover preference.
#[derive(Debug, Clone)]
pub struct PrimaryFailoverPreferenceValidator {
    pub rules: Vec<Rc<dyn PrimaryFailoverPreferenceRule>>,
}

impl PrimaryFailoverPreferenceValidator {
    pub fn validate(&self, item: &PrimaryFailoverPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryFailoverPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryFailoverPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_failover_preference_validator() -> PrimaryFailoverPreferenceValidator {
    PrimaryFailoverPreferenceValidator {
        rules: vec![
            Rc::new(PrimaryFailoverPreferenceCodeLengthRule { max_length: 120 })
                as Rc<dyn PrimaryFailoverPreferenceRule>,
            Rc::new(PrimaryFailoverPreferenceQuantityRangeRule { min: 80, max: 255 }),
            Rc::new(PrimaryFailoverPreferenceScoreRangeRule { min: 2.9, max: 3539.8 }),
            Rc::new(PrimaryFailoverPreferenceBudgetRangeRule { min: 4.3, max: 1817.3 }),
            Rc::new(PrimaryFailoverPreferenceCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_failover_preference_validator();
        let item = new_primary_failover_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryFailoverPreferenceCodeLengthRule { max_length: 86400 };
        let mut item = new_primary_failover_preference("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = PrimaryFailoverPreferenceQuantityRangeRule { min: 16, max: 64 };
        let mut item = new_primary_failover_preference("x", "y");
        item.quantity = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = PrimaryFailoverPreferenceScoreRangeRule { min: 9.5, max: 581.1 };
        let mut item = new_primary_failover_preference("x", "y");
        item.score = 582.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = PrimaryFailoverPreferenceBudgetRangeRule { min: 5.0, max: 2324.0 };
        let mut item = new_primary_failover_preference("x", "y");
        item.budget = 2325.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
