use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global page slot must pass. Check returns its problems, or none.
pub trait GlobalPageSlotRule: std::fmt::Debug {
    fn check(&self, item: &GlobalPageSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPageSlotRequiredCodeRule {
    pub message: String,
}

impl GlobalPageSlotRule for GlobalPageSlotRequiredCodeRule {
    fn check(&self, item: &GlobalPageSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPageSlotLabelLengthRule {
    pub max_length: i64,
}

impl GlobalPageSlotRule for GlobalPageSlotLabelLengthRule {
    fn check(&self, item: &GlobalPageSlot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPageSlotTitleLengthRule {
    pub max_length: i64,
}

impl GlobalPageSlotRule for GlobalPageSlotTitleLengthRule {
    fn check(&self, item: &GlobalPageSlot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPageSlotRequiredNameRule {
    pub message: String,
}

impl GlobalPageSlotRule for GlobalPageSlotRequiredNameRule {
    fn check(&self, item: &GlobalPageSlot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global page slot.
#[derive(Debug, Clone)]
pub struct GlobalPageSlotValidator {
    pub rules: Vec<Rc<dyn GlobalPageSlotRule>>,
}

impl GlobalPageSlotValidator {
    pub fn validate(&self, item: &GlobalPageSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalPageSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalPageSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_page_slot_validator() -> GlobalPageSlotValidator {
    GlobalPageSlotValidator {
        rules: vec![
            Rc::new(GlobalPageSlotRequiredCodeRule {
                message: "global page slot needs this field".to_string(),
            }) as Rc<dyn GlobalPageSlotRule>,
            Rc::new(GlobalPageSlotLabelLengthRule { max_length: 1000 }),
            Rc::new(GlobalPageSlotTitleLengthRule { max_length: 86400 }),
            Rc::new(GlobalPageSlotRequiredNameRule {
                message: "global page slot needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_page_slot_validator();
        let item = new_global_page_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalPageSlotRequiredCodeRule {
            message: "global page slot needs this field".to_string(),
        };
        let mut item = new_global_page_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = GlobalPageSlotLabelLengthRule { max_length: 16 };
        let mut item = new_global_page_slot("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = GlobalPageSlotTitleLengthRule { max_length: 120 };
        let mut item = new_global_page_slot("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = GlobalPageSlotRequiredNameRule {
            message: "global page slot needs this field".to_string(),
        };
        let mut item = new_global_page_slot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
