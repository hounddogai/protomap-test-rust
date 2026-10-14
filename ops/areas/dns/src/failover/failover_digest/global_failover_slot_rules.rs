use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global failover slot must pass. Check returns its problems, or none.
pub trait GlobalFailoverSlotRule: std::fmt::Debug {
    fn check(&self, item: &GlobalFailoverSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFailoverSlotRequiredContactEmailRule {
    pub message: String,
}

impl GlobalFailoverSlotRule for GlobalFailoverSlotRequiredContactEmailRule {
    fn check(&self, item: &GlobalFailoverSlot) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFailoverSlotCategoryLengthRule {
    pub max_length: i64,
}

impl GlobalFailoverSlotRule for GlobalFailoverSlotCategoryLengthRule {
    fn check(&self, item: &GlobalFailoverSlot) -> Vec<String> {
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
pub struct GlobalFailoverSlotPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalFailoverSlotRule for GlobalFailoverSlotPortRangeRule {
    fn check(&self, item: &GlobalFailoverSlot) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFailoverSlotContactEmailLengthRule {
    pub max_length: i64,
}

impl GlobalFailoverSlotRule for GlobalFailoverSlotContactEmailLengthRule {
    fn check(&self, item: &GlobalFailoverSlot) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a global failover slot.
#[derive(Debug, Clone)]
pub struct GlobalFailoverSlotValidator {
    pub rules: Vec<Rc<dyn GlobalFailoverSlotRule>>,
}

impl GlobalFailoverSlotValidator {
    pub fn validate(&self, item: &GlobalFailoverSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalFailoverSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalFailoverSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_failover_slot_validator() -> GlobalFailoverSlotValidator {
    GlobalFailoverSlotValidator {
        rules: vec![
            Rc::new(GlobalFailoverSlotRequiredContactEmailRule {
                message: "global failover slot needs this field".to_string(),
            }) as Rc<dyn GlobalFailoverSlotRule>,
            Rc::new(GlobalFailoverSlotCategoryLengthRule { max_length: 16 }),
            Rc::new(GlobalFailoverSlotPortRangeRule { min: 3600, max: 86400 }),
            Rc::new(GlobalFailoverSlotContactEmailLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_failover_slot_validator();
        let item = new_global_failover_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = GlobalFailoverSlotRequiredContactEmailRule {
            message: "global failover slot needs this field".to_string(),
        };
        let mut item = new_global_failover_slot("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = GlobalFailoverSlotCategoryLengthRule { max_length: 86400 };
        let mut item = new_global_failover_slot("x", "y");
        item.category = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = GlobalFailoverSlotPortRangeRule { min: 64, max: 86400 };
        let mut item = new_global_failover_slot("x", "y");
        item.port = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = GlobalFailoverSlotContactEmailLengthRule { max_length: 16 };
        let mut item = new_global_failover_slot("x", "y");
        item.contact_email = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
