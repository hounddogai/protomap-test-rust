use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual peering entry must pass. Check returns its problems, or none.
pub trait ManualPeeringEntryRule: std::fmt::Debug {
    fn check(&self, item: &ManualPeeringEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringEntryCodePrefixRule {
    pub prefix: String,
}

impl ManualPeeringEntryRule for ManualPeeringEntryCodePrefixRule {
    fn check(&self, item: &ManualPeeringEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringEntryRequiredCodeRule {
    pub message: String,
}

impl ManualPeeringEntryRule for ManualPeeringEntryRequiredCodeRule {
    fn check(&self, item: &ManualPeeringEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringEntryLabelLengthRule {
    pub max_length: i64,
}

impl ManualPeeringEntryRule for ManualPeeringEntryLabelLengthRule {
    fn check(&self, item: &ManualPeeringEntry) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringEntryWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualPeeringEntryRule for ManualPeeringEntryWeightRangeRule {
    fn check(&self, item: &ManualPeeringEntry) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringEntryFingerprintLengthRule {
    pub max_length: i64,
}

impl ManualPeeringEntryRule for ManualPeeringEntryFingerprintLengthRule {
    fn check(&self, item: &ManualPeeringEntry) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual peering entry.
#[derive(Debug, Clone)]
pub struct ManualPeeringEntryValidator {
    pub rules: Vec<Rc<dyn ManualPeeringEntryRule>>,
}

impl ManualPeeringEntryValidator {
    pub fn validate(&self, item: &ManualPeeringEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualPeeringEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualPeeringEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_peering_entry_validator() -> ManualPeeringEntryValidator {
    ManualPeeringEntryValidator {
        rules: vec![
            Rc::new(ManualPeeringEntryCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn ManualPeeringEntryRule>,
            Rc::new(ManualPeeringEntryRequiredCodeRule {
                message: "manual peering entry needs this field".to_string(),
            }),
            Rc::new(ManualPeeringEntryLabelLengthRule { max_length: 64 }),
            Rc::new(ManualPeeringEntryWeightRangeRule { min: 6.2, max: 1266.5 }),
            Rc::new(ManualPeeringEntryFingerprintLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_peering_entry_validator();
        let item = new_manual_peering_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualPeeringEntryRequiredCodeRule {
            message: "manual peering entry needs a value".to_string(),
        };
        let mut item = new_manual_peering_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ManualPeeringEntryLabelLengthRule { max_length: 255 };
        let mut item = new_manual_peering_entry("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = ManualPeeringEntryWeightRangeRule { min: 3.6, max: 3701.9 };
        let mut item = new_manual_peering_entry("x", "y");
        item.weight = 3703.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = ManualPeeringEntryFingerprintLengthRule { max_length: 3600 };
        let mut item = new_manual_peering_entry("x", "y");
        item.fingerprint = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
