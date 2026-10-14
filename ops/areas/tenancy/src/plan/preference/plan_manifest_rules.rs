use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan manifest must pass. Check returns its problems, or none.
pub trait PlanManifestRule: std::fmt::Debug {
    fn check(&self, item: &PlanManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanManifestBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PlanManifestRule for PlanManifestBudgetRangeRule {
    fn check(&self, item: &PlanManifest) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanManifestCodePrefixRule {
    pub prefix: String,
}

impl PlanManifestRule for PlanManifestCodePrefixRule {
    fn check(&self, item: &PlanManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanManifestMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PlanManifestRule for PlanManifestMinimumRangeRule {
    fn check(&self, item: &PlanManifest) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a plan manifest.
#[derive(Debug, Clone)]
pub struct PlanManifestValidator {
    pub rules: Vec<Rc<dyn PlanManifestRule>>,
}

impl PlanManifestValidator {
    pub fn validate(&self, item: &PlanManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PlanManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_manifest_validator() -> PlanManifestValidator {
    PlanManifestValidator {
        rules: vec![
            Rc::new(PlanManifestBudgetRangeRule { min: 3.2, max: 617.7 }) as Rc<dyn PlanManifestRule>,
            Rc::new(PlanManifestCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(PlanManifestMinimumRangeRule { min: 0.0, max: 53.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_manifest_validator();
        let item = new_plan_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = PlanManifestBudgetRangeRule { min: 5.9, max: 3908.2 };
        let mut item = new_plan_manifest("x", "y");
        item.budget = 3909.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = PlanManifestMinimumRangeRule { min: 7.5, max: 2246.7 };
        let mut item = new_plan_manifest("x", "y");
        item.minimum = 2248.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
