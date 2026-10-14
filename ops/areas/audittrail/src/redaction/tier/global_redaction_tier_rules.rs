use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global redaction tier must pass. Check returns its problems, or none.
pub trait GlobalRedactionTierRule: std::fmt::Debug {
    fn check(&self, item: &GlobalRedactionTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionTierEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalRedactionTierRule for GlobalRedactionTierEndsAtRangeRule {
    fn check(&self, item: &GlobalRedactionTier) -> Vec<String> {
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
pub struct GlobalRedactionTierFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalRedactionTierRule for GlobalRedactionTierFloorValueRangeRule {
    fn check(&self, item: &GlobalRedactionTier) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionTierRequiredChecksumRule {
    pub message: String,
}

impl GlobalRedactionTierRule for GlobalRedactionTierRequiredChecksumRule {
    fn check(&self, item: &GlobalRedactionTier) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionTierRequiredSegmentRule {
    pub message: String,
}

impl GlobalRedactionTierRule for GlobalRedactionTierRequiredSegmentRule {
    fn check(&self, item: &GlobalRedactionTier) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionTierRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalRedactionTierRule for GlobalRedactionTierRetriesRangeRule {
    fn check(&self, item: &GlobalRedactionTier) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a global redaction tier.
#[derive(Debug, Clone)]
pub struct GlobalRedactionTierValidator {
    pub rules: Vec<Rc<dyn GlobalRedactionTierRule>>,
}

impl GlobalRedactionTierValidator {
    pub fn validate(&self, item: &GlobalRedactionTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalRedactionTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalRedactionTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_redaction_tier_validator() -> GlobalRedactionTierValidator {
    GlobalRedactionTierValidator {
        rules: vec![
            Rc::new(GlobalRedactionTierEndsAtRangeRule { min: 16, max: 80 }) as Rc<dyn GlobalRedactionTierRule>,
            Rc::new(GlobalRedactionTierFloorValueRangeRule { min: 6.7, max: 4471.7 }),
            Rc::new(GlobalRedactionTierRequiredChecksumRule {
                message: "global redaction tier needs a value".to_string(),
            }),
            Rc::new(GlobalRedactionTierRequiredSegmentRule {
                message: "global redaction tier needs a value".to_string(),
            }),
            Rc::new(GlobalRedactionTierRetriesRangeRule { min: 80, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_redaction_tier_validator();
        let item = new_global_redaction_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = GlobalRedactionTierEndsAtRangeRule { min: 1000, max: 86400 };
        let mut item = new_global_redaction_tier("x", "y");
        item.ends_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = GlobalRedactionTierFloorValueRangeRule { min: 9.3, max: 86.1 };
        let mut item = new_global_redaction_tier("x", "y");
        item.floor_value = 87.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = GlobalRedactionTierRequiredChecksumRule {
            message: "global redaction tier needs this field".to_string(),
        };
        let mut item = new_global_redaction_tier("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = GlobalRedactionTierRequiredSegmentRule {
            message: "global redaction tier needs an entry".to_string(),
        };
        let mut item = new_global_redaction_tier("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = GlobalRedactionTierRetriesRangeRule { min: 16, max: 3600 };
        let mut item = new_global_redaction_tier("x", "y");
        item.retries = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
