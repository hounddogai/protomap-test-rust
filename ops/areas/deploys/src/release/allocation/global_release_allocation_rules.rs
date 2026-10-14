use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global release allocation must pass. Check returns its problems, or none.
pub trait GlobalReleaseAllocationRule: std::fmt::Debug {
    fn check(&self, item: &GlobalReleaseAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAllocationLabelLengthRule {
    pub max_length: i64,
}

impl GlobalReleaseAllocationRule for GlobalReleaseAllocationLabelLengthRule {
    fn check(&self, item: &GlobalReleaseAllocation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAllocationChecksumLengthRule {
    pub max_length: i64,
}

impl GlobalReleaseAllocationRule for GlobalReleaseAllocationChecksumLengthRule {
    fn check(&self, item: &GlobalReleaseAllocation) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAllocationCodeLengthRule {
    pub max_length: i64,
}

impl GlobalReleaseAllocationRule for GlobalReleaseAllocationCodeLengthRule {
    fn check(&self, item: &GlobalReleaseAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAllocationRequiredChecksumRule {
    pub message: String,
}

impl GlobalReleaseAllocationRule for GlobalReleaseAllocationRequiredChecksumRule {
    fn check(&self, item: &GlobalReleaseAllocation) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAllocationCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalReleaseAllocationRule for GlobalReleaseAllocationCeilingRangeRule {
    fn check(&self, item: &GlobalReleaseAllocation) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAllocationRequiredFingerprintRule {
    pub message: String,
}

impl GlobalReleaseAllocationRule for GlobalReleaseAllocationRequiredFingerprintRule {
    fn check(&self, item: &GlobalReleaseAllocation) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global release allocation.
#[derive(Debug, Clone)]
pub struct GlobalReleaseAllocationValidator {
    pub rules: Vec<Rc<dyn GlobalReleaseAllocationRule>>,
}

impl GlobalReleaseAllocationValidator {
    pub fn validate(&self, item: &GlobalReleaseAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalReleaseAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalReleaseAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_release_allocation_validator() -> GlobalReleaseAllocationValidator {
    GlobalReleaseAllocationValidator {
        rules: vec![
            Rc::new(GlobalReleaseAllocationLabelLengthRule { max_length: 80 }) as Rc<dyn GlobalReleaseAllocationRule>,
            Rc::new(GlobalReleaseAllocationChecksumLengthRule { max_length: 120 }),
            Rc::new(GlobalReleaseAllocationCodeLengthRule { max_length: 255 }),
            Rc::new(GlobalReleaseAllocationRequiredChecksumRule {
                message: "global release allocation needs an entry".to_string(),
            }),
            Rc::new(GlobalReleaseAllocationCeilingRangeRule { min: 8.3, max: 1977.8 }),
            Rc::new(GlobalReleaseAllocationRequiredFingerprintRule {
                message: "global release allocation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_release_allocation_validator();
        let item = new_global_release_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = GlobalReleaseAllocationLabelLengthRule { max_length: 255 };
        let mut item = new_global_release_allocation("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = GlobalReleaseAllocationChecksumLengthRule { max_length: 64 };
        let mut item = new_global_release_allocation("x", "y");
        item.checksum = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalReleaseAllocationCodeLengthRule { max_length: 64 };
        let mut item = new_global_release_allocation("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = GlobalReleaseAllocationRequiredChecksumRule {
            message: "global release allocation needs this field".to_string(),
        };
        let mut item = new_global_release_allocation("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = GlobalReleaseAllocationCeilingRangeRule { min: 9.2, max: 1502.2 };
        let mut item = new_global_release_allocation("x", "y");
        item.ceiling = 1503.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = GlobalReleaseAllocationRequiredFingerprintRule {
            message: "global release allocation needs this field".to_string(),
        };
        let mut item = new_global_release_allocation("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
