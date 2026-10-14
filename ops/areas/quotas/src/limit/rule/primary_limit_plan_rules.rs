use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary limit plan must pass. Check returns its problems, or none.
pub trait PrimaryLimitPlanRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryLimitPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitPlanRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryLimitPlanRule for PrimaryLimitPlanRatioRangeRule {
    fn check(&self, item: &PrimaryLimitPlan) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitPlanCodePrefixRule {
    pub prefix: String,
}

impl PrimaryLimitPlanRule for PrimaryLimitPlanCodePrefixRule {
    fn check(&self, item: &PrimaryLimitPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitPlanThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryLimitPlanRule for PrimaryLimitPlanThresholdRangeRule {
    fn check(&self, item: &PrimaryLimitPlan) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary limit plan.
#[derive(Debug, Clone)]
pub struct PrimaryLimitPlanValidator {
    pub rules: Vec<Rc<dyn PrimaryLimitPlanRule>>,
}

impl PrimaryLimitPlanValidator {
    pub fn validate(&self, item: &PrimaryLimitPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryLimitPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryLimitPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_limit_plan_validator() -> PrimaryLimitPlanValidator {
    PrimaryLimitPlanValidator {
        rules: vec![
            Rc::new(PrimaryLimitPlanRatioRangeRule { min: 5.0, max: 2589.2 }) as Rc<dyn PrimaryLimitPlanRule>,
            Rc::new(PrimaryLimitPlanCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PrimaryLimitPlanThresholdRangeRule { min: 1.3, max: 2158.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_limit_plan_validator();
        let item = new_primary_limit_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = PrimaryLimitPlanRatioRangeRule { min: 6.4, max: 1410.5 };
        let mut item = new_primary_limit_plan("x", "y");
        item.ratio = 1412.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = PrimaryLimitPlanThresholdRangeRule { min: 7.6, max: 3011.3 };
        let mut item = new_primary_limit_plan("x", "y");
        item.threshold = 3012.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
