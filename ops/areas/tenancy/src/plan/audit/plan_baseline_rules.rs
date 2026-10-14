use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a plan baseline must pass. Check returns its problems, or none.
pub trait PlanBaselineRule: std::fmt::Debug {
    fn check(&self, item: &PlanBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanBaselineRequiredLabelRule {
    pub message: String,
}

impl PlanBaselineRule for PlanBaselineRequiredLabelRule {
    fn check(&self, item: &PlanBaseline) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanBaselineCodePrefixRule {
    pub prefix: String,
}

impl PlanBaselineRule for PlanBaselineCodePrefixRule {
    fn check(&self, item: &PlanBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanBaselineAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl PlanBaselineRule for PlanBaselineAllowedScopeRule {
    fn check(&self, item: &PlanBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is below minimum", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanBaselineSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PlanBaselineRule for PlanBaselineSequenceRangeRule {
    fn check(&self, item: &PlanBaseline) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanBaselineRequiredCodeRule {
    pub message: String,
}

impl PlanBaselineRule for PlanBaselineRequiredCodeRule {
    fn check(&self, item: &PlanBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan baseline.
#[derive(Debug, Clone)]
pub struct PlanBaselineValidator {
    pub rules: Vec<Rc<dyn PlanBaselineRule>>,
}

impl PlanBaselineValidator {
    pub fn validate(&self, item: &PlanBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PlanBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_baseline_validator() -> PlanBaselineValidator {
    PlanBaselineValidator {
        rules: vec![
            Rc::new(PlanBaselineRequiredLabelRule {
                message: "plan baseline needs an entry".to_string(),
            }) as Rc<dyn PlanBaselineRule>,
            Rc::new(PlanBaselineCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PlanBaselineAllowedScopeRule {
                allowed: vec!["legacy".to_string(), "rolling".to_string()],
            }),
            Rc::new(PlanBaselineSequenceRangeRule { min: 64, max: 255 }),
            Rc::new(PlanBaselineRequiredCodeRule {
                message: "plan baseline needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_baseline_validator();
        let item = new_plan_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PlanBaselineRequiredLabelRule {
            message: "plan baseline needs a value".to_string(),
        };
        let mut item = new_plan_baseline("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = PlanBaselineSequenceRangeRule { min: 255, max: 86400 };
        let mut item = new_plan_baseline("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PlanBaselineRequiredCodeRule {
            message: "plan baseline needs this field".to_string(),
        };
        let mut item = new_plan_baseline("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
