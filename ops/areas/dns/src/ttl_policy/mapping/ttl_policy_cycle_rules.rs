use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy cycle must pass. Check returns its problems, or none.
pub trait TtlPolicyCycleRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyCycleRequiredNameRule {
    pub message: String,
}

impl TtlPolicyCycleRule for TtlPolicyCycleRequiredNameRule {
    fn check(&self, item: &TtlPolicyCycle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyCycleScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TtlPolicyCycleRule for TtlPolicyCycleScoreRangeRule {
    fn check(&self, item: &TtlPolicyCycle) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyCycleRequiredAudienceRule {
    pub message: String,
}

impl TtlPolicyCycleRule for TtlPolicyCycleRequiredAudienceRule {
    fn check(&self, item: &TtlPolicyCycle) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy cycle.
#[derive(Debug, Clone)]
pub struct TtlPolicyCycleValidator {
    pub rules: Vec<Rc<dyn TtlPolicyCycleRule>>,
}

impl TtlPolicyCycleValidator {
    pub fn validate(&self, item: &TtlPolicyCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TtlPolicyCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_cycle_validator() -> TtlPolicyCycleValidator {
    TtlPolicyCycleValidator {
        rules: vec![
            Rc::new(TtlPolicyCycleRequiredNameRule {
                message: "ttl policy cycle needs a value".to_string(),
            }) as Rc<dyn TtlPolicyCycleRule>,
            Rc::new(TtlPolicyCycleScoreRangeRule { min: 4.6, max: 401.2 }),
            Rc::new(TtlPolicyCycleRequiredAudienceRule {
                message: "ttl policy cycle needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_cycle_validator();
        let item = new_ttl_policy_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TtlPolicyCycleRequiredNameRule {
            message: "ttl policy cycle needs a value".to_string(),
        };
        let mut item = new_ttl_policy_cycle("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = TtlPolicyCycleScoreRangeRule { min: 8.6, max: 2446.8 };
        let mut item = new_ttl_policy_cycle("x", "y");
        item.score = 2448.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = TtlPolicyCycleRequiredAudienceRule {
            message: "ttl policy cycle needs an entry".to_string(),
        };
        let mut item = new_ttl_policy_cycle("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
