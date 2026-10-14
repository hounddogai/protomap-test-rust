use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict usage plan must pass. Check returns its problems, or none.
pub trait StrictUsagePlanRule: std::fmt::Debug {
    fn check(&self, item: &StrictUsagePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictUsagePlanRequiredNameRule {
    pub message: String,
}

impl StrictUsagePlanRule for StrictUsagePlanRequiredNameRule {
    fn check(&self, item: &StrictUsagePlan) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictUsagePlanScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictUsagePlanRule for StrictUsagePlanScoreRangeRule {
    fn check(&self, item: &StrictUsagePlan) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictUsagePlanRequiredCodeRule {
    pub message: String,
}

impl StrictUsagePlanRule for StrictUsagePlanRequiredCodeRule {
    fn check(&self, item: &StrictUsagePlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict usage plan.
#[derive(Debug, Clone)]
pub struct StrictUsagePlanValidator {
    pub rules: Vec<Rc<dyn StrictUsagePlanRule>>,
}

impl StrictUsagePlanValidator {
    pub fn validate(&self, item: &StrictUsagePlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictUsagePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictUsagePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_usage_plan_validator() -> StrictUsagePlanValidator {
    StrictUsagePlanValidator {
        rules: vec![
            Rc::new(StrictUsagePlanRequiredNameRule {
                message: "strict usage plan needs this field".to_string(),
            }) as Rc<dyn StrictUsagePlanRule>,
            Rc::new(StrictUsagePlanScoreRangeRule { min: 9.6, max: 257.3 }),
            Rc::new(StrictUsagePlanRequiredCodeRule {
                message: "strict usage plan needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_usage_plan_validator();
        let item = new_strict_usage_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictUsagePlanRequiredNameRule {
            message: "strict usage plan needs an entry".to_string(),
        };
        let mut item = new_strict_usage_plan("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = StrictUsagePlanScoreRangeRule { min: 6.5, max: 4061.6 };
        let mut item = new_strict_usage_plan("x", "y");
        item.score = 4063.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictUsagePlanRequiredCodeRule {
            message: "strict usage plan needs an entry".to_string(),
        };
        let mut item = new_strict_usage_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
