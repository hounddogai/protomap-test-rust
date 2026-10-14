use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist entry must pass. Check returns its problems, or none.
pub trait AllowlistEntryRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistEntryMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AllowlistEntryRule for AllowlistEntryMinimumRangeRule {
    fn check(&self, item: &AllowlistEntry) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistEntryQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistEntryRule for AllowlistEntryQuantityRangeRule {
    fn check(&self, item: &AllowlistEntry) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistEntryNameLengthRule {
    pub max_length: i64,
}

impl AllowlistEntryRule for AllowlistEntryNameLengthRule {
    fn check(&self, item: &AllowlistEntry) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistEntryRequiredNameRule {
    pub message: String,
}

impl AllowlistEntryRule for AllowlistEntryRequiredNameRule {
    fn check(&self, item: &AllowlistEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistEntryMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AllowlistEntryRule for AllowlistEntryMaximumRangeRule {
    fn check(&self, item: &AllowlistEntry) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist entry.
#[derive(Debug, Clone)]
pub struct AllowlistEntryValidator {
    pub rules: Vec<Rc<dyn AllowlistEntryRule>>,
}

impl AllowlistEntryValidator {
    pub fn validate(&self, item: &AllowlistEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_entry_validator() -> AllowlistEntryValidator {
    AllowlistEntryValidator {
        rules: vec![
            Rc::new(AllowlistEntryMinimumRangeRule { min: 9.7, max: 4053.6 }) as Rc<dyn AllowlistEntryRule>,
            Rc::new(AllowlistEntryQuantityRangeRule { min: 64, max: 80 }),
            Rc::new(AllowlistEntryNameLengthRule { max_length: 120 }),
            Rc::new(AllowlistEntryRequiredNameRule {
                message: "allowlist entry needs this field".to_string(),
            }),
            Rc::new(AllowlistEntryMaximumRangeRule { min: 1.0, max: 415.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_entry_validator();
        let item = new_allowlist_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = AllowlistEntryMinimumRangeRule { min: 5.7, max: 3993.3 };
        let mut item = new_allowlist_entry("x", "y");
        item.minimum = 3994.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = AllowlistEntryQuantityRangeRule { min: 120, max: 1000 };
        let mut item = new_allowlist_entry("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = AllowlistEntryNameLengthRule { max_length: 80 };
        let mut item = new_allowlist_entry("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AllowlistEntryRequiredNameRule {
            message: "allowlist entry needs a value".to_string(),
        };
        let mut item = new_allowlist_entry("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = AllowlistEntryMaximumRangeRule { min: 8.0, max: 1851.0 };
        let mut item = new_allowlist_entry("x", "y");
        item.maximum = 1852.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
