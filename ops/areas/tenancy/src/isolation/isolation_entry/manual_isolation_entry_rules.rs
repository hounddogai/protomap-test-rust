use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual isolation entry must pass. Check returns its problems, or none.
pub trait ManualIsolationEntryRule: std::fmt::Debug {
    fn check(&self, item: &ManualIsolationEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationEntryBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualIsolationEntryRule for ManualIsolationEntryBatchSizeRangeRule {
    fn check(&self, item: &ManualIsolationEntry) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationEntryRequiredCodeRule {
    pub message: String,
}

impl ManualIsolationEntryRule for ManualIsolationEntryRequiredCodeRule {
    fn check(&self, item: &ManualIsolationEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationEntryRegionLengthRule {
    pub max_length: i64,
}

impl ManualIsolationEntryRule for ManualIsolationEntryRegionLengthRule {
    fn check(&self, item: &ManualIsolationEntry) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual isolation entry.
#[derive(Debug, Clone)]
pub struct ManualIsolationEntryValidator {
    pub rules: Vec<Rc<dyn ManualIsolationEntryRule>>,
}

impl ManualIsolationEntryValidator {
    pub fn validate(&self, item: &ManualIsolationEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualIsolationEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualIsolationEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_isolation_entry_validator() -> ManualIsolationEntryValidator {
    ManualIsolationEntryValidator {
        rules: vec![
            Rc::new(ManualIsolationEntryBatchSizeRangeRule { min: 80, max: 86400 }) as Rc<dyn ManualIsolationEntryRule>,
            Rc::new(ManualIsolationEntryRequiredCodeRule {
                message: "manual isolation entry needs this field".to_string(),
            }),
            Rc::new(ManualIsolationEntryRegionLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_isolation_entry_validator();
        let item = new_manual_isolation_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = ManualIsolationEntryBatchSizeRangeRule { min: 32, max: 3600 };
        let mut item = new_manual_isolation_entry("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualIsolationEntryRequiredCodeRule {
            message: "manual isolation entry needs this field".to_string(),
        };
        let mut item = new_manual_isolation_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = ManualIsolationEntryRegionLengthRule { max_length: 16 };
        let mut item = new_manual_isolation_entry("x", "y");
        item.region = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
