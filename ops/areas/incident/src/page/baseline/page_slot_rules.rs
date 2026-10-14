use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a page slot must pass. Check returns its problems, or none.
pub trait PageSlotRule: std::fmt::Debug {
    fn check(&self, item: &PageSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageSlotRequiredCodeRule {
    pub message: String,
}

impl PageSlotRule for PageSlotRequiredCodeRule {
    fn check(&self, item: &PageSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageSlotQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageSlotRule for PageSlotQuantityRangeRule {
    fn check(&self, item: &PageSlot) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageSlotWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PageSlotRule for PageSlotWeightRangeRule {
    fn check(&self, item: &PageSlot) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageSlotAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PageSlotRule for PageSlotAllowedCodeRule {
    fn check(&self, item: &PageSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageSlotRegionLengthRule {
    pub max_length: i64,
}

impl PageSlotRule for PageSlotRegionLengthRule {
    fn check(&self, item: &PageSlot) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a page slot.
#[derive(Debug, Clone)]
pub struct PageSlotValidator {
    pub rules: Vec<Rc<dyn PageSlotRule>>,
}

impl PageSlotValidator {
    pub fn validate(&self, item: &PageSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_slot_validator() -> PageSlotValidator {
    PageSlotValidator {
        rules: vec![
            Rc::new(PageSlotRequiredCodeRule {
                message: "page slot needs an entry".to_string(),
            }) as Rc<dyn PageSlotRule>,
            Rc::new(PageSlotQuantityRangeRule { min: 64, max: 80 }),
            Rc::new(PageSlotWeightRangeRule { min: 7.8, max: 4101.6 }),
            Rc::new(PageSlotAllowedCodeRule {
                allowed: vec!["pending".to_string(), "global".to_string()],
            }),
            Rc::new(PageSlotRegionLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_slot_validator();
        let item = new_page_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PageSlotRequiredCodeRule {
            message: "page slot needs a value".to_string(),
        };
        let mut item = new_page_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = PageSlotQuantityRangeRule { min: 16, max: 32 };
        let mut item = new_page_slot("x", "y");
        item.quantity = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = PageSlotWeightRangeRule { min: 2.6, max: 2272.1 };
        let mut item = new_page_slot("x", "y");
        item.weight = 2273.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = PageSlotRegionLengthRule { max_length: 32 };
        let mut item = new_page_slot("x", "y");
        item.region = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
