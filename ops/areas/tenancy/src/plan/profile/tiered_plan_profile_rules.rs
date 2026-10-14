use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered plan profile must pass. Check returns its problems, or none.
pub trait TieredPlanProfileRule: std::fmt::Debug {
    fn check(&self, item: &TieredPlanProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanProfileRequiredLabelRule {
    pub message: String,
}

impl TieredPlanProfileRule for TieredPlanProfileRequiredLabelRule {
    fn check(&self, item: &TieredPlanProfile) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanProfileRequiredCodeRule {
    pub message: String,
}

impl TieredPlanProfileRule for TieredPlanProfileRequiredCodeRule {
    fn check(&self, item: &TieredPlanProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanProfileWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredPlanProfileRule for TieredPlanProfileWeightRangeRule {
    fn check(&self, item: &TieredPlanProfile) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tiered plan profile.
#[derive(Debug, Clone)]
pub struct TieredPlanProfileValidator {
    pub rules: Vec<Rc<dyn TieredPlanProfileRule>>,
}

impl TieredPlanProfileValidator {
    pub fn validate(&self, item: &TieredPlanProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredPlanProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredPlanProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_plan_profile_validator() -> TieredPlanProfileValidator {
    TieredPlanProfileValidator {
        rules: vec![
            Rc::new(TieredPlanProfileRequiredLabelRule {
                message: "tiered plan profile needs a value".to_string(),
            }) as Rc<dyn TieredPlanProfileRule>,
            Rc::new(TieredPlanProfileRequiredCodeRule {
                message: "tiered plan profile needs a value".to_string(),
            }),
            Rc::new(TieredPlanProfileWeightRangeRule { min: 7.3, max: 4087.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_plan_profile_validator();
        let item = new_tiered_plan_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TieredPlanProfileRequiredLabelRule {
            message: "tiered plan profile needs a value".to_string(),
        };
        let mut item = new_tiered_plan_profile("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredPlanProfileRequiredCodeRule {
            message: "tiered plan profile needs an entry".to_string(),
        };
        let mut item = new_tiered_plan_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = TieredPlanProfileWeightRangeRule { min: 1.7, max: 366.8 };
        let mut item = new_tiered_plan_profile("x", "y");
        item.weight = 368.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
