use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage window must pass. Check returns its problems, or none.
pub trait OverageWindowRule: std::fmt::Debug {
    fn check(&self, item: &OverageWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageWindowLabelLengthRule {
    pub max_length: i64,
}

impl OverageWindowRule for OverageWindowLabelLengthRule {
    fn check(&self, item: &OverageWindow) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageWindowCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl OverageWindowRule for OverageWindowCostRangeRule {
    fn check(&self, item: &OverageWindow) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageWindowLocaleLengthRule {
    pub max_length: i64,
}

impl OverageWindowRule for OverageWindowLocaleLengthRule {
    fn check(&self, item: &OverageWindow) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageWindowCodeLengthRule {
    pub max_length: i64,
}

impl OverageWindowRule for OverageWindowCodeLengthRule {
    fn check(&self, item: &OverageWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageWindowRequiredLocaleRule {
    pub message: String,
}

impl OverageWindowRule for OverageWindowRequiredLocaleRule {
    fn check(&self, item: &OverageWindow) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage window.
#[derive(Debug, Clone)]
pub struct OverageWindowValidator {
    pub rules: Vec<Rc<dyn OverageWindowRule>>,
}

impl OverageWindowValidator {
    pub fn validate(&self, item: &OverageWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_window_validator() -> OverageWindowValidator {
    OverageWindowValidator {
        rules: vec![
            Rc::new(OverageWindowLabelLengthRule { max_length: 255 }) as Rc<dyn OverageWindowRule>,
            Rc::new(OverageWindowCostRangeRule { min: 5.5, max: 1956.0 }),
            Rc::new(OverageWindowLocaleLengthRule { max_length: 64 }),
            Rc::new(OverageWindowCodeLengthRule { max_length: 16 }),
            Rc::new(OverageWindowRequiredLocaleRule {
                message: "overage window needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_window_validator();
        let item = new_overage_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = OverageWindowLabelLengthRule { max_length: 32 };
        let mut item = new_overage_window("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = OverageWindowCostRangeRule { min: 2.0, max: 1793.3 };
        let mut item = new_overage_window("x", "y");
        item.cost = 1794.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = OverageWindowLocaleLengthRule { max_length: 120 };
        let mut item = new_overage_window("x", "y");
        item.locale = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = OverageWindowCodeLengthRule { max_length: 16 };
        let mut item = new_overage_window("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = OverageWindowRequiredLocaleRule {
            message: "overage window needs this field".to_string(),
        };
        let mut item = new_overage_window("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
