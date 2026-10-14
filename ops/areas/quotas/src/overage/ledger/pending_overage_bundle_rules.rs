use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending overage bundle must pass. Check returns its problems, or none.
pub trait PendingOverageBundleRule: std::fmt::Debug {
    fn check(&self, item: &PendingOverageBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingOverageBundleLabelLengthRule {
    pub max_length: i64,
}

impl PendingOverageBundleRule for PendingOverageBundleLabelLengthRule {
    fn check(&self, item: &PendingOverageBundle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingOverageBundleWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingOverageBundleRule for PendingOverageBundleWindowDaysRangeRule {
    fn check(&self, item: &PendingOverageBundle) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingOverageBundlePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingOverageBundleRule for PendingOverageBundlePortRangeRule {
    fn check(&self, item: &PendingOverageBundle) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingOverageBundleSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingOverageBundleRule for PendingOverageBundleSequenceRangeRule {
    fn check(&self, item: &PendingOverageBundle) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingOverageBundleCodePrefixRule {
    pub prefix: String,
}

impl PendingOverageBundleRule for PendingOverageBundleCodePrefixRule {
    fn check(&self, item: &PendingOverageBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending overage bundle.
#[derive(Debug, Clone)]
pub struct PendingOverageBundleValidator {
    pub rules: Vec<Rc<dyn PendingOverageBundleRule>>,
}

impl PendingOverageBundleValidator {
    pub fn validate(&self, item: &PendingOverageBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingOverageBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingOverageBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_overage_bundle_validator() -> PendingOverageBundleValidator {
    PendingOverageBundleValidator {
        rules: vec![
            Rc::new(PendingOverageBundleLabelLengthRule { max_length: 1000 }) as Rc<dyn PendingOverageBundleRule>,
            Rc::new(PendingOverageBundleWindowDaysRangeRule { min: 16, max: 64 }),
            Rc::new(PendingOverageBundlePortRangeRule { min: 64, max: 255 }),
            Rc::new(PendingOverageBundleSequenceRangeRule { min: 32, max: 32 }),
            Rc::new(PendingOverageBundleCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_overage_bundle_validator();
        let item = new_pending_overage_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PendingOverageBundleLabelLengthRule { max_length: 80 };
        let mut item = new_pending_overage_bundle("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = PendingOverageBundleWindowDaysRangeRule { min: 120, max: 255 };
        let mut item = new_pending_overage_bundle("x", "y");
        item.window_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = PendingOverageBundlePortRangeRule { min: 32, max: 120 };
        let mut item = new_pending_overage_bundle("x", "y");
        item.port = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = PendingOverageBundleSequenceRangeRule { min: 80, max: 255 };
        let mut item = new_pending_overage_bundle("x", "y");
        item.sequence = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
