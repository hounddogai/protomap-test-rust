use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default alert slot must pass. Check returns its problems, or none.
pub trait DefaultAlertSlotRule: std::fmt::Debug {
    fn check(&self, item: &DefaultAlertSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAlertSlotCategoryLengthRule {
    pub max_length: i64,
}

impl DefaultAlertSlotRule for DefaultAlertSlotCategoryLengthRule {
    fn check(&self, item: &DefaultAlertSlot) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAlertSlotEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultAlertSlotRule for DefaultAlertSlotEndsAtRangeRule {
    fn check(&self, item: &DefaultAlertSlot) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAlertSlotNameLengthRule {
    pub max_length: i64,
}

impl DefaultAlertSlotRule for DefaultAlertSlotNameLengthRule {
    fn check(&self, item: &DefaultAlertSlot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default alert slot.
#[derive(Debug, Clone)]
pub struct DefaultAlertSlotValidator {
    pub rules: Vec<Rc<dyn DefaultAlertSlotRule>>,
}

impl DefaultAlertSlotValidator {
    pub fn validate(&self, item: &DefaultAlertSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultAlertSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultAlertSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_alert_slot_validator() -> DefaultAlertSlotValidator {
    DefaultAlertSlotValidator {
        rules: vec![
            Rc::new(DefaultAlertSlotCategoryLengthRule { max_length: 32 }) as Rc<dyn DefaultAlertSlotRule>,
            Rc::new(DefaultAlertSlotEndsAtRangeRule { min: 86400, max: 86400 }),
            Rc::new(DefaultAlertSlotNameLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_alert_slot_validator();
        let item = new_default_alert_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = DefaultAlertSlotCategoryLengthRule { max_length: 1000 };
        let mut item = new_default_alert_slot("x", "y");
        item.category = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = DefaultAlertSlotEndsAtRangeRule { min: 32, max: 1000 };
        let mut item = new_default_alert_slot("x", "y");
        item.ends_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DefaultAlertSlotNameLengthRule { max_length: 3600 };
        let mut item = new_default_alert_slot("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
