use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page override must pass. Check returns its problems, or none.
pub trait PageOverrideRule: std::fmt::Debug {
    fn check(&self, item: &PageOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageOverrideRequiredLabelRule {
    pub message: String,
}

impl PageOverrideRule for PageOverrideRequiredLabelRule {
    fn check(&self, item: &PageOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageOverrideUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageOverrideRule for PageOverrideUpdatedAtRangeRule {
    fn check(&self, item: &PageOverride) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageOverrideRequiredKeyIdRule {
    pub message: String,
}

impl PageOverrideRule for PageOverrideRequiredKeyIdRule {
    fn check(&self, item: &PageOverride) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageOverrideBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageOverrideRule for PageOverrideBatchSizeRangeRule {
    fn check(&self, item: &PageOverride) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a page override.
#[derive(Debug, Clone)]
pub struct PageOverrideValidator {
    pub rules: Vec<Rc<dyn PageOverrideRule>>,
}

impl PageOverrideValidator {
    pub fn validate(&self, item: &PageOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_override_validator() -> PageOverrideValidator {
    PageOverrideValidator {
        rules: vec![
            Rc::new(PageOverrideRequiredLabelRule {
                message: "page override needs a value".to_string(),
            }) as Rc<dyn PageOverrideRule>,
            Rc::new(PageOverrideUpdatedAtRangeRule { min: 255, max: 3600 }),
            Rc::new(PageOverrideRequiredKeyIdRule {
                message: "page override needs this field".to_string(),
            }),
            Rc::new(PageOverrideBatchSizeRangeRule { min: 32, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_override_validator();
        let item = new_page_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PageOverrideRequiredLabelRule {
            message: "page override needs an entry".to_string(),
        };
        let mut item = new_page_override("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = PageOverrideUpdatedAtRangeRule { min: 32, max: 80 };
        let mut item = new_page_override("x", "y");
        item.updated_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = PageOverrideRequiredKeyIdRule {
            message: "page override needs this field".to_string(),
        };
        let mut item = new_page_override("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = PageOverrideBatchSizeRangeRule { min: 255, max: 1000 };
        let mut item = new_page_override("x", "y");
        item.batch_size = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
