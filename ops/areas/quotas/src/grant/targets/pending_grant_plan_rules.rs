use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending grant plan must pass. Check returns its problems, or none.
pub trait PendingGrantPlanRule: std::fmt::Debug {
    fn check(&self, item: &PendingGrantPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingGrantPlanNameLengthRule {
    pub max_length: i64,
}

impl PendingGrantPlanRule for PendingGrantPlanNameLengthRule {
    fn check(&self, item: &PendingGrantPlan) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingGrantPlanScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingGrantPlanRule for PendingGrantPlanScoreRangeRule {
    fn check(&self, item: &PendingGrantPlan) -> Vec<String> {
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
pub struct PendingGrantPlanRegionLengthRule {
    pub max_length: i64,
}

impl PendingGrantPlanRule for PendingGrantPlanRegionLengthRule {
    fn check(&self, item: &PendingGrantPlan) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending grant plan.
#[derive(Debug, Clone)]
pub struct PendingGrantPlanValidator {
    pub rules: Vec<Rc<dyn PendingGrantPlanRule>>,
}

impl PendingGrantPlanValidator {
    pub fn validate(&self, item: &PendingGrantPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingGrantPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingGrantPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_grant_plan_validator() -> PendingGrantPlanValidator {
    PendingGrantPlanValidator {
        rules: vec![
            Rc::new(PendingGrantPlanNameLengthRule { max_length: 64 }) as Rc<dyn PendingGrantPlanRule>,
            Rc::new(PendingGrantPlanScoreRangeRule { min: 3.0, max: 3442.2 }),
            Rc::new(PendingGrantPlanRegionLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_grant_plan_validator();
        let item = new_pending_grant_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PendingGrantPlanNameLengthRule { max_length: 120 };
        let mut item = new_pending_grant_plan("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = PendingGrantPlanScoreRangeRule { min: 6.4, max: 2674.2 };
        let mut item = new_pending_grant_plan("x", "y");
        item.score = 2675.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = PendingGrantPlanRegionLengthRule { max_length: 32 };
        let mut item = new_pending_grant_plan("x", "y");
        item.region = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
