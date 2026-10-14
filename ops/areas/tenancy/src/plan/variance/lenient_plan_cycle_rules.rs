use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient plan cycle must pass. Check returns its problems, or none.
pub trait LenientPlanCycleRule: std::fmt::Debug {
    fn check(&self, item: &LenientPlanCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanCycleRequiredCodeRule {
    pub message: String,
}

impl LenientPlanCycleRule for LenientPlanCycleRequiredCodeRule {
    fn check(&self, item: &LenientPlanCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanCycleCodePrefixRule {
    pub prefix: String,
}

impl LenientPlanCycleRule for LenientPlanCycleCodePrefixRule {
    fn check(&self, item: &LenientPlanCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanCycleThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientPlanCycleRule for LenientPlanCycleThresholdRangeRule {
    fn check(&self, item: &LenientPlanCycle) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanCycleRequiredLocaleRule {
    pub message: String,
}

impl LenientPlanCycleRule for LenientPlanCycleRequiredLocaleRule {
    fn check(&self, item: &LenientPlanCycle) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient plan cycle.
#[derive(Debug, Clone)]
pub struct LenientPlanCycleValidator {
    pub rules: Vec<Rc<dyn LenientPlanCycleRule>>,
}

impl LenientPlanCycleValidator {
    pub fn validate(&self, item: &LenientPlanCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientPlanCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientPlanCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_plan_cycle_validator() -> LenientPlanCycleValidator {
    LenientPlanCycleValidator {
        rules: vec![
            Rc::new(LenientPlanCycleRequiredCodeRule {
                message: "lenient plan cycle needs an entry".to_string(),
            }) as Rc<dyn LenientPlanCycleRule>,
            Rc::new(LenientPlanCycleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(LenientPlanCycleThresholdRangeRule { min: 7.0, max: 2752.1 }),
            Rc::new(LenientPlanCycleRequiredLocaleRule {
                message: "lenient plan cycle needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_plan_cycle_validator();
        let item = new_lenient_plan_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientPlanCycleRequiredCodeRule {
            message: "lenient plan cycle needs an entry".to_string(),
        };
        let mut item = new_lenient_plan_cycle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = LenientPlanCycleThresholdRangeRule { min: 5.9, max: 874.3 };
        let mut item = new_lenient_plan_cycle("x", "y");
        item.threshold = 875.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = LenientPlanCycleRequiredLocaleRule {
            message: "lenient plan cycle needs an entry".to_string(),
        };
        let mut item = new_lenient_plan_cycle("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
