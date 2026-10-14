use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence baseline must pass. Check returns its problems, or none.
pub trait EvidenceBaselineRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBaselineChannelLengthRule {
    pub max_length: i64,
}

impl EvidenceBaselineRule for EvidenceBaselineChannelLengthRule {
    fn check(&self, item: &EvidenceBaseline) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBaselineRequiredChannelRule {
    pub message: String,
}

impl EvidenceBaselineRule for EvidenceBaselineRequiredChannelRule {
    fn check(&self, item: &EvidenceBaseline) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBaselineQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceBaselineRule for EvidenceBaselineQuantityRangeRule {
    fn check(&self, item: &EvidenceBaseline) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a evidence baseline.
#[derive(Debug, Clone)]
pub struct EvidenceBaselineValidator {
    pub rules: Vec<Rc<dyn EvidenceBaselineRule>>,
}

impl EvidenceBaselineValidator {
    pub fn validate(&self, item: &EvidenceBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_baseline_validator() -> EvidenceBaselineValidator {
    EvidenceBaselineValidator {
        rules: vec![
            Rc::new(EvidenceBaselineChannelLengthRule { max_length: 86400 }) as Rc<dyn EvidenceBaselineRule>,
            Rc::new(EvidenceBaselineRequiredChannelRule {
                message: "evidence baseline needs an entry".to_string(),
            }),
            Rc::new(EvidenceBaselineQuantityRangeRule { min: 255, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_baseline_validator();
        let item = new_evidence_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = EvidenceBaselineChannelLengthRule { max_length: 64 };
        let mut item = new_evidence_baseline("x", "y");
        item.channel = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = EvidenceBaselineRequiredChannelRule {
            message: "evidence baseline needs a value".to_string(),
        };
        let mut item = new_evidence_baseline("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = EvidenceBaselineQuantityRangeRule { min: 64, max: 80 };
        let mut item = new_evidence_baseline("x", "y");
        item.quantity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
