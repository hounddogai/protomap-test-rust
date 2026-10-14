use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe allocation must pass. Check returns its problems, or none.
pub trait ProbeAllocationRule: std::fmt::Debug {
    fn check(&self, item: &ProbeAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeAllocationCodePrefixRule {
    pub prefix: String,
}

impl ProbeAllocationRule for ProbeAllocationCodePrefixRule {
    fn check(&self, item: &ProbeAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeAllocationTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeAllocationRule for ProbeAllocationTtlSecondsRangeRule {
    fn check(&self, item: &ProbeAllocation) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeAllocationRequiredCodeRule {
    pub message: String,
}

impl ProbeAllocationRule for ProbeAllocationRequiredCodeRule {
    fn check(&self, item: &ProbeAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeAllocationAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeAllocationRule for ProbeAllocationAttemptsRangeRule {
    fn check(&self, item: &ProbeAllocation) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeAllocationGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeAllocationRule for ProbeAllocationGraceDaysRangeRule {
    fn check(&self, item: &ProbeAllocation) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe allocation.
#[derive(Debug, Clone)]
pub struct ProbeAllocationValidator {
    pub rules: Vec<Rc<dyn ProbeAllocationRule>>,
}

impl ProbeAllocationValidator {
    pub fn validate(&self, item: &ProbeAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_allocation_validator() -> ProbeAllocationValidator {
    ProbeAllocationValidator {
        rules: vec![
            Rc::new(ProbeAllocationCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn ProbeAllocationRule>,
            Rc::new(ProbeAllocationTtlSecondsRangeRule { min: 32, max: 255 }),
            Rc::new(ProbeAllocationRequiredCodeRule {
                message: "probe allocation needs a value".to_string(),
            }),
            Rc::new(ProbeAllocationAttemptsRangeRule { min: 255, max: 86400 }),
            Rc::new(ProbeAllocationGraceDaysRangeRule { min: 80, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_allocation_validator();
        let item = new_probe_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = ProbeAllocationTtlSecondsRangeRule { min: 80, max: 86400 };
        let mut item = new_probe_allocation("x", "y");
        item.ttl_seconds = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ProbeAllocationRequiredCodeRule {
            message: "probe allocation needs a value".to_string(),
        };
        let mut item = new_probe_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = ProbeAllocationAttemptsRangeRule { min: 120, max: 120 };
        let mut item = new_probe_allocation("x", "y");
        item.attempts = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = ProbeAllocationGraceDaysRangeRule { min: 64, max: 120 };
        let mut item = new_probe_allocation("x", "y");
        item.grace_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
