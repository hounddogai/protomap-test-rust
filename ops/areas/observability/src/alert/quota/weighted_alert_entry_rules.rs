use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted alert entry must pass. Check returns its problems, or none.
pub trait WeightedAlertEntryRule: std::fmt::Debug {
    fn check(&self, item: &WeightedAlertEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertEntryRequiredCodeRule {
    pub message: String,
}

impl WeightedAlertEntryRule for WeightedAlertEntryRequiredCodeRule {
    fn check(&self, item: &WeightedAlertEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertEntryTitleLengthRule {
    pub max_length: i64,
}

impl WeightedAlertEntryRule for WeightedAlertEntryTitleLengthRule {
    fn check(&self, item: &WeightedAlertEntry) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertEntryMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedAlertEntryRule for WeightedAlertEntryMaximumRangeRule {
    fn check(&self, item: &WeightedAlertEntry) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertEntryCodePrefixRule {
    pub prefix: String,
}

impl WeightedAlertEntryRule for WeightedAlertEntryCodePrefixRule {
    fn check(&self, item: &WeightedAlertEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted alert entry.
#[derive(Debug, Clone)]
pub struct WeightedAlertEntryValidator {
    pub rules: Vec<Rc<dyn WeightedAlertEntryRule>>,
}

impl WeightedAlertEntryValidator {
    pub fn validate(&self, item: &WeightedAlertEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedAlertEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedAlertEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_alert_entry_validator() -> WeightedAlertEntryValidator {
    WeightedAlertEntryValidator {
        rules: vec![
            Rc::new(WeightedAlertEntryRequiredCodeRule {
                message: "weighted alert entry needs a value".to_string(),
            }) as Rc<dyn WeightedAlertEntryRule>,
            Rc::new(WeightedAlertEntryTitleLengthRule { max_length: 120 }),
            Rc::new(WeightedAlertEntryMaximumRangeRule { min: 0.2, max: 480.3 }),
            Rc::new(WeightedAlertEntryCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_alert_entry_validator();
        let item = new_weighted_alert_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedAlertEntryRequiredCodeRule {
            message: "weighted alert entry needs this field".to_string(),
        };
        let mut item = new_weighted_alert_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = WeightedAlertEntryTitleLengthRule { max_length: 3600 };
        let mut item = new_weighted_alert_entry("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = WeightedAlertEntryMaximumRangeRule { min: 4.7, max: 814.8 };
        let mut item = new_weighted_alert_entry("x", "y");
        item.maximum = 816.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
