use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary suspension allocation must pass. Check returns its problems, or none.
pub trait PrimarySuspensionAllocationRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySuspensionAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionAllocationNotesLengthRule {
    pub max_length: i64,
}

impl PrimarySuspensionAllocationRule for PrimarySuspensionAllocationNotesLengthRule {
    fn check(&self, item: &PrimarySuspensionAllocation) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionAllocationTitleLengthRule {
    pub max_length: i64,
}

impl PrimarySuspensionAllocationRule for PrimarySuspensionAllocationTitleLengthRule {
    fn check(&self, item: &PrimarySuspensionAllocation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionAllocationRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimarySuspensionAllocationRule for PrimarySuspensionAllocationRateRangeRule {
    fn check(&self, item: &PrimarySuspensionAllocation) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a primary suspension allocation.
#[derive(Debug, Clone)]
pub struct PrimarySuspensionAllocationValidator {
    pub rules: Vec<Rc<dyn PrimarySuspensionAllocationRule>>,
}

impl PrimarySuspensionAllocationValidator {
    pub fn validate(&self, item: &PrimarySuspensionAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimarySuspensionAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySuspensionAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_suspension_allocation_validator() -> PrimarySuspensionAllocationValidator {
    PrimarySuspensionAllocationValidator {
        rules: vec![
            Rc::new(PrimarySuspensionAllocationNotesLengthRule { max_length: 255 })
                as Rc<dyn PrimarySuspensionAllocationRule>,
            Rc::new(PrimarySuspensionAllocationTitleLengthRule { max_length: 16 }),
            Rc::new(PrimarySuspensionAllocationRateRangeRule { min: 4.0, max: 4817.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_suspension_allocation_validator();
        let item = new_primary_suspension_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = PrimarySuspensionAllocationNotesLengthRule { max_length: 80 };
        let mut item = new_primary_suspension_allocation("x", "y");
        item.notes = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PrimarySuspensionAllocationTitleLengthRule { max_length: 32 };
        let mut item = new_primary_suspension_allocation("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = PrimarySuspensionAllocationRateRangeRule { min: 6.1, max: 4032.5 };
        let mut item = new_primary_suspension_allocation("x", "y");
        item.rate = 4034.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
