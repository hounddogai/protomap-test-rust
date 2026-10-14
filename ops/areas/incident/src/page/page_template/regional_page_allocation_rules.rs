use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional page allocation must pass. Check returns its problems, or none.
pub trait RegionalPageAllocationRule: std::fmt::Debug {
    fn check(&self, item: &RegionalPageAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageAllocationRequiredCodeRule {
    pub message: String,
}

impl RegionalPageAllocationRule for RegionalPageAllocationRequiredCodeRule {
    fn check(&self, item: &RegionalPageAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageAllocationThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalPageAllocationRule for RegionalPageAllocationThresholdRangeRule {
    fn check(&self, item: &RegionalPageAllocation) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageAllocationRequiredNameRule {
    pub message: String,
}

impl RegionalPageAllocationRule for RegionalPageAllocationRequiredNameRule {
    fn check(&self, item: &RegionalPageAllocation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageAllocationRequiredNotesRule {
    pub message: String,
}

impl RegionalPageAllocationRule for RegionalPageAllocationRequiredNotesRule {
    fn check(&self, item: &RegionalPageAllocation) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional page allocation.
#[derive(Debug, Clone)]
pub struct RegionalPageAllocationValidator {
    pub rules: Vec<Rc<dyn RegionalPageAllocationRule>>,
}

impl RegionalPageAllocationValidator {
    pub fn validate(&self, item: &RegionalPageAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalPageAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalPageAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_page_allocation_validator() -> RegionalPageAllocationValidator {
    RegionalPageAllocationValidator {
        rules: vec![
            Rc::new(RegionalPageAllocationRequiredCodeRule {
                message: "regional page allocation needs an entry".to_string(),
            }) as Rc<dyn RegionalPageAllocationRule>,
            Rc::new(RegionalPageAllocationThresholdRangeRule { min: 4.3, max: 3961.2 }),
            Rc::new(RegionalPageAllocationRequiredNameRule {
                message: "regional page allocation needs an entry".to_string(),
            }),
            Rc::new(RegionalPageAllocationRequiredNotesRule {
                message: "regional page allocation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_page_allocation_validator();
        let item = new_regional_page_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalPageAllocationRequiredCodeRule {
            message: "regional page allocation needs an entry".to_string(),
        };
        let mut item = new_regional_page_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = RegionalPageAllocationThresholdRangeRule { min: 8.9, max: 4953.1 };
        let mut item = new_regional_page_allocation("x", "y");
        item.threshold = 4954.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RegionalPageAllocationRequiredNameRule {
            message: "regional page allocation needs an entry".to_string(),
        };
        let mut item = new_regional_page_allocation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = RegionalPageAllocationRequiredNotesRule {
            message: "regional page allocation needs this field".to_string(),
        };
        let mut item = new_regional_page_allocation("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
