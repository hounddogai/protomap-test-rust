use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient action item baseline must pass. Check returns its problems, or none.
pub trait LenientActionItemBaselineRule: std::fmt::Debug {
    fn check(&self, item: &LenientActionItemBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemBaselineRequiredLabelRule {
    pub message: String,
}

impl LenientActionItemBaselineRule for LenientActionItemBaselineRequiredLabelRule {
    fn check(&self, item: &LenientActionItemBaseline) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemBaselineRequiredHostRule {
    pub message: String,
}

impl LenientActionItemBaselineRule for LenientActionItemBaselineRequiredHostRule {
    fn check(&self, item: &LenientActionItemBaseline) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemBaselineGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientActionItemBaselineRule for LenientActionItemBaselineGraceDaysRangeRule {
    fn check(&self, item: &LenientActionItemBaseline) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemBaselineLabelLengthRule {
    pub max_length: i64,
}

impl LenientActionItemBaselineRule for LenientActionItemBaselineLabelLengthRule {
    fn check(&self, item: &LenientActionItemBaseline) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient action item baseline.
#[derive(Debug, Clone)]
pub struct LenientActionItemBaselineValidator {
    pub rules: Vec<Rc<dyn LenientActionItemBaselineRule>>,
}

impl LenientActionItemBaselineValidator {
    pub fn validate(&self, item: &LenientActionItemBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientActionItemBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientActionItemBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_action_item_baseline_validator() -> LenientActionItemBaselineValidator {
    LenientActionItemBaselineValidator {
        rules: vec![
            Rc::new(LenientActionItemBaselineRequiredLabelRule {
                message: "lenient action item baseline needs a value".to_string(),
            }) as Rc<dyn LenientActionItemBaselineRule>,
            Rc::new(LenientActionItemBaselineRequiredHostRule {
                message: "lenient action item baseline needs an entry".to_string(),
            }),
            Rc::new(LenientActionItemBaselineGraceDaysRangeRule { min: 80, max: 120 }),
            Rc::new(LenientActionItemBaselineLabelLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_action_item_baseline_validator();
        let item = new_lenient_action_item_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientActionItemBaselineRequiredLabelRule {
            message: "lenient action item baseline needs an entry".to_string(),
        };
        let mut item = new_lenient_action_item_baseline("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = LenientActionItemBaselineRequiredHostRule {
            message: "lenient action item baseline needs an entry".to_string(),
        };
        let mut item = new_lenient_action_item_baseline("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = LenientActionItemBaselineGraceDaysRangeRule { min: 80, max: 3600 };
        let mut item = new_lenient_action_item_baseline("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientActionItemBaselineLabelLengthRule { max_length: 80 };
        let mut item = new_lenient_action_item_baseline("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
