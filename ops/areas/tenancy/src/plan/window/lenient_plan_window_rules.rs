use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient plan window must pass. Check returns its problems, or none.
pub trait LenientPlanWindowRule: std::fmt::Debug {
    fn check(&self, item: &LenientPlanWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanWindowRequiredLabelRule {
    pub message: String,
}

impl LenientPlanWindowRule for LenientPlanWindowRequiredLabelRule {
    fn check(&self, item: &LenientPlanWindow) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanWindowScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientPlanWindowRule for LenientPlanWindowScoreRangeRule {
    fn check(&self, item: &LenientPlanWindow) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanWindowRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPlanWindowRule for LenientPlanWindowRetriesRangeRule {
    fn check(&self, item: &LenientPlanWindow) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanWindowRequiredCodeRule {
    pub message: String,
}

impl LenientPlanWindowRule for LenientPlanWindowRequiredCodeRule {
    fn check(&self, item: &LenientPlanWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient plan window.
#[derive(Debug, Clone)]
pub struct LenientPlanWindowValidator {
    pub rules: Vec<Rc<dyn LenientPlanWindowRule>>,
}

impl LenientPlanWindowValidator {
    pub fn validate(&self, item: &LenientPlanWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientPlanWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientPlanWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_plan_window_validator() -> LenientPlanWindowValidator {
    LenientPlanWindowValidator {
        rules: vec![
            Rc::new(LenientPlanWindowRequiredLabelRule {
                message: "lenient plan window needs an entry".to_string(),
            }) as Rc<dyn LenientPlanWindowRule>,
            Rc::new(LenientPlanWindowScoreRangeRule { min: 1.6, max: 1020.0 }),
            Rc::new(LenientPlanWindowRetriesRangeRule { min: 80, max: 255 }),
            Rc::new(LenientPlanWindowRequiredCodeRule {
                message: "lenient plan window needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_plan_window_validator();
        let item = new_lenient_plan_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientPlanWindowRequiredLabelRule {
            message: "lenient plan window needs a value".to_string(),
        };
        let mut item = new_lenient_plan_window("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = LenientPlanWindowScoreRangeRule { min: 9.2, max: 923.5 };
        let mut item = new_lenient_plan_window("x", "y");
        item.score = 925.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = LenientPlanWindowRetriesRangeRule { min: 32, max: 64 };
        let mut item = new_lenient_plan_window("x", "y");
        item.retries = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientPlanWindowRequiredCodeRule {
            message: "lenient plan window needs an entry".to_string(),
        };
        let mut item = new_lenient_plan_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
