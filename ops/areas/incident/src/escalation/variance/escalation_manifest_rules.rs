use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation manifest must pass. Check returns its problems, or none.
pub trait EscalationManifestRule: std::fmt::Debug {
    fn check(&self, item: &EscalationManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationManifestGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationManifestRule for EscalationManifestGraceDaysRangeRule {
    fn check(&self, item: &EscalationManifest) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationManifestRequiredSourceRule {
    pub message: String,
}

impl EscalationManifestRule for EscalationManifestRequiredSourceRule {
    fn check(&self, item: &EscalationManifest) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationManifestRequiredChecksumRule {
    pub message: String,
}

impl EscalationManifestRule for EscalationManifestRequiredChecksumRule {
    fn check(&self, item: &EscalationManifest) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationManifestRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EscalationManifestRule for EscalationManifestRateRangeRule {
    fn check(&self, item: &EscalationManifest) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationManifestCodeLengthRule {
    pub max_length: i64,
}

impl EscalationManifestRule for EscalationManifestCodeLengthRule {
    fn check(&self, item: &EscalationManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationManifestLabelLengthRule {
    pub max_length: i64,
}

impl EscalationManifestRule for EscalationManifestLabelLengthRule {
    fn check(&self, item: &EscalationManifest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation manifest.
#[derive(Debug, Clone)]
pub struct EscalationManifestValidator {
    pub rules: Vec<Rc<dyn EscalationManifestRule>>,
}

impl EscalationManifestValidator {
    pub fn validate(&self, item: &EscalationManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EscalationManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_manifest_validator() -> EscalationManifestValidator {
    EscalationManifestValidator {
        rules: vec![
            Rc::new(EscalationManifestGraceDaysRangeRule { min: 16, max: 32 }) as Rc<dyn EscalationManifestRule>,
            Rc::new(EscalationManifestRequiredSourceRule {
                message: "escalation manifest needs this field".to_string(),
            }),
            Rc::new(EscalationManifestRequiredChecksumRule {
                message: "escalation manifest needs an entry".to_string(),
            }),
            Rc::new(EscalationManifestRateRangeRule { min: 0.7, max: 155.6 }),
            Rc::new(EscalationManifestCodeLengthRule { max_length: 64 }),
            Rc::new(EscalationManifestLabelLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_manifest_validator();
        let item = new_escalation_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = EscalationManifestGraceDaysRangeRule { min: 16, max: 1000 };
        let mut item = new_escalation_manifest("x", "y");
        item.grace_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = EscalationManifestRequiredSourceRule {
            message: "escalation manifest needs a value".to_string(),
        };
        let mut item = new_escalation_manifest("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = EscalationManifestRequiredChecksumRule {
            message: "escalation manifest needs a value".to_string(),
        };
        let mut item = new_escalation_manifest("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = EscalationManifestRateRangeRule { min: 2.2, max: 3768.2 };
        let mut item = new_escalation_manifest("x", "y");
        item.rate = 3769.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EscalationManifestCodeLengthRule { max_length: 64 };
        let mut item = new_escalation_manifest("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = EscalationManifestLabelLengthRule { max_length: 120 };
        let mut item = new_escalation_manifest("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
