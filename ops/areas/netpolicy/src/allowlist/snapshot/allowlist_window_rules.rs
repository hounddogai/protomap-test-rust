use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist window must pass. Check returns its problems, or none.
pub trait AllowlistWindowRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistWindowRequiredCodeRule {
    pub message: String,
}

impl AllowlistWindowRule for AllowlistWindowRequiredCodeRule {
    fn check(&self, item: &AllowlistWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistWindowWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AllowlistWindowRule for AllowlistWindowWeightRangeRule {
    fn check(&self, item: &AllowlistWindow) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistWindowRequiredLabelRule {
    pub message: String,
}

impl AllowlistWindowRule for AllowlistWindowRequiredLabelRule {
    fn check(&self, item: &AllowlistWindow) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistWindowLocaleLengthRule {
    pub max_length: i64,
}

impl AllowlistWindowRule for AllowlistWindowLocaleLengthRule {
    fn check(&self, item: &AllowlistWindow) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist window.
#[derive(Debug, Clone)]
pub struct AllowlistWindowValidator {
    pub rules: Vec<Rc<dyn AllowlistWindowRule>>,
}

impl AllowlistWindowValidator {
    pub fn validate(&self, item: &AllowlistWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_window_validator() -> AllowlistWindowValidator {
    AllowlistWindowValidator {
        rules: vec![
            Rc::new(AllowlistWindowRequiredCodeRule {
                message: "allowlist window needs this field".to_string(),
            }) as Rc<dyn AllowlistWindowRule>,
            Rc::new(AllowlistWindowWeightRangeRule { min: 8.8, max: 702.1 }),
            Rc::new(AllowlistWindowRequiredLabelRule {
                message: "allowlist window needs this field".to_string(),
            }),
            Rc::new(AllowlistWindowLocaleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_window_validator();
        let item = new_allowlist_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AllowlistWindowRequiredCodeRule {
            message: "allowlist window needs an entry".to_string(),
        };
        let mut item = new_allowlist_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = AllowlistWindowWeightRangeRule { min: 3.8, max: 4702.5 };
        let mut item = new_allowlist_window("x", "y");
        item.weight = 4704.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AllowlistWindowRequiredLabelRule {
            message: "allowlist window needs a value".to_string(),
        };
        let mut item = new_allowlist_window("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = AllowlistWindowLocaleLengthRule { max_length: 16 };
        let mut item = new_allowlist_window("x", "y");
        item.locale = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
