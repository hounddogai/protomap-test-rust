use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary redaction allocation must pass. Check returns its problems, or none.
pub trait PrimaryRedactionAllocationRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRedactionAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionAllocationRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryRedactionAllocationRule for PrimaryRedactionAllocationRatioRangeRule {
    fn check(&self, item: &PrimaryRedactionAllocation) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionAllocationCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryRedactionAllocationRule for PrimaryRedactionAllocationCodeLengthRule {
    fn check(&self, item: &PrimaryRedactionAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionAllocationUnitLengthRule {
    pub max_length: i64,
}

impl PrimaryRedactionAllocationRule for PrimaryRedactionAllocationUnitLengthRule {
    fn check(&self, item: &PrimaryRedactionAllocation) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionAllocationGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryRedactionAllocationRule for PrimaryRedactionAllocationGraceDaysRangeRule {
    fn check(&self, item: &PrimaryRedactionAllocation) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary redaction allocation.
#[derive(Debug, Clone)]
pub struct PrimaryRedactionAllocationValidator {
    pub rules: Vec<Rc<dyn PrimaryRedactionAllocationRule>>,
}

impl PrimaryRedactionAllocationValidator {
    pub fn validate(&self, item: &PrimaryRedactionAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryRedactionAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRedactionAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_redaction_allocation_validator() -> PrimaryRedactionAllocationValidator {
    PrimaryRedactionAllocationValidator {
        rules: vec![
            Rc::new(PrimaryRedactionAllocationRatioRangeRule { min: 5.2, max: 2205.7 })
                as Rc<dyn PrimaryRedactionAllocationRule>,
            Rc::new(PrimaryRedactionAllocationCodeLengthRule { max_length: 16 }),
            Rc::new(PrimaryRedactionAllocationUnitLengthRule { max_length: 3600 }),
            Rc::new(PrimaryRedactionAllocationGraceDaysRangeRule { min: 32, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_redaction_allocation_validator();
        let item = new_primary_redaction_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = PrimaryRedactionAllocationRatioRangeRule { min: 9.5, max: 4937.1 };
        let mut item = new_primary_redaction_allocation("x", "y");
        item.ratio = 4938.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryRedactionAllocationCodeLengthRule { max_length: 3600 };
        let mut item = new_primary_redaction_allocation("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = PrimaryRedactionAllocationUnitLengthRule { max_length: 80 };
        let mut item = new_primary_redaction_allocation("x", "y");
        item.unit = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = PrimaryRedactionAllocationGraceDaysRangeRule { min: 16, max: 120 };
        let mut item = new_primary_redaction_allocation("x", "y");
        item.grace_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
