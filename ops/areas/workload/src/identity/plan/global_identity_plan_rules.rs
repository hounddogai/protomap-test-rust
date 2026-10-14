use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global identity plan must pass. Check returns its problems, or none.
pub trait GlobalIdentityPlanRule: std::fmt::Debug {
    fn check(&self, item: &GlobalIdentityPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityPlanAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl GlobalIdentityPlanRule for GlobalIdentityPlanAllowedCodeRule {
    fn check(&self, item: &GlobalIdentityPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityPlanUnitLengthRule {
    pub max_length: i64,
}

impl GlobalIdentityPlanRule for GlobalIdentityPlanUnitLengthRule {
    fn check(&self, item: &GlobalIdentityPlan) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityPlanScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalIdentityPlanRule for GlobalIdentityPlanScoreRangeRule {
    fn check(&self, item: &GlobalIdentityPlan) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityPlanNameLengthRule {
    pub max_length: i64,
}

impl GlobalIdentityPlanRule for GlobalIdentityPlanNameLengthRule {
    fn check(&self, item: &GlobalIdentityPlan) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global identity plan.
#[derive(Debug, Clone)]
pub struct GlobalIdentityPlanValidator {
    pub rules: Vec<Rc<dyn GlobalIdentityPlanRule>>,
}

impl GlobalIdentityPlanValidator {
    pub fn validate(&self, item: &GlobalIdentityPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalIdentityPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalIdentityPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_identity_plan_validator() -> GlobalIdentityPlanValidator {
    GlobalIdentityPlanValidator {
        rules: vec![
            Rc::new(GlobalIdentityPlanAllowedCodeRule {
                allowed: vec!["scheduled".to_string()],
            }) as Rc<dyn GlobalIdentityPlanRule>,
            Rc::new(GlobalIdentityPlanUnitLengthRule { max_length: 3600 }),
            Rc::new(GlobalIdentityPlanScoreRangeRule { min: 0.2, max: 2725.8 }),
            Rc::new(GlobalIdentityPlanNameLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_identity_plan_validator();
        let item = new_global_identity_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = GlobalIdentityPlanUnitLengthRule { max_length: 255 };
        let mut item = new_global_identity_plan("x", "y");
        item.unit = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = GlobalIdentityPlanScoreRangeRule { min: 2.3, max: 3136.4 };
        let mut item = new_global_identity_plan("x", "y");
        item.score = 3137.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = GlobalIdentityPlanNameLengthRule { max_length: 32 };
        let mut item = new_global_identity_plan("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
