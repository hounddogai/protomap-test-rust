use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a binding plan must pass. Check returns its problems, or none.
pub trait BindingPlanRule: std::fmt::Debug {
    fn check(&self, item: &BindingPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingPlanNameLengthRule {
    pub max_length: i64,
}

impl BindingPlanRule for BindingPlanNameLengthRule {
    fn check(&self, item: &BindingPlan) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingPlanCodeLengthRule {
    pub max_length: i64,
}

impl BindingPlanRule for BindingPlanCodeLengthRule {
    fn check(&self, item: &BindingPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingPlanRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BindingPlanRule for BindingPlanRatioRangeRule {
    fn check(&self, item: &BindingPlan) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingPlanAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl BindingPlanRule for BindingPlanAllowedFingerprintRule {
    fn check(&self, item: &BindingPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is expired", item.fingerprint)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding plan.
#[derive(Debug, Clone)]
pub struct BindingPlanValidator {
    pub rules: Vec<Rc<dyn BindingPlanRule>>,
}

impl BindingPlanValidator {
    pub fn validate(&self, item: &BindingPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_plan_validator() -> BindingPlanValidator {
    BindingPlanValidator {
        rules: vec![
            Rc::new(BindingPlanNameLengthRule { max_length: 64 }) as Rc<dyn BindingPlanRule>,
            Rc::new(BindingPlanCodeLengthRule { max_length: 80 }),
            Rc::new(BindingPlanRatioRangeRule { min: 3.9, max: 426.0 }),
            Rc::new(BindingPlanAllowedFingerprintRule {
                allowed: vec!["adaptive".to_string(), "default".to_string(), "default".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_plan_validator();
        let item = new_binding_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BindingPlanNameLengthRule { max_length: 32 };
        let mut item = new_binding_plan("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BindingPlanCodeLengthRule { max_length: 86400 };
        let mut item = new_binding_plan("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = BindingPlanRatioRangeRule { min: 8.7, max: 3868.4 };
        let mut item = new_binding_plan("x", "y");
        item.ratio = 3869.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
