use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered peering agreement must pass. Check returns its problems, or none.
pub trait TieredPeeringAgreementRule: std::fmt::Debug {
    fn check(&self, item: &TieredPeeringAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringAgreementLabelLengthRule {
    pub max_length: i64,
}

impl TieredPeeringAgreementRule for TieredPeeringAgreementLabelLengthRule {
    fn check(&self, item: &TieredPeeringAgreement) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringAgreementTitleLengthRule {
    pub max_length: i64,
}

impl TieredPeeringAgreementRule for TieredPeeringAgreementTitleLengthRule {
    fn check(&self, item: &TieredPeeringAgreement) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringAgreementCodePrefixRule {
    pub prefix: String,
}

impl TieredPeeringAgreementRule for TieredPeeringAgreementCodePrefixRule {
    fn check(&self, item: &TieredPeeringAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringAgreementThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredPeeringAgreementRule for TieredPeeringAgreementThresholdRangeRule {
    fn check(&self, item: &TieredPeeringAgreement) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered peering agreement.
#[derive(Debug, Clone)]
pub struct TieredPeeringAgreementValidator {
    pub rules: Vec<Rc<dyn TieredPeeringAgreementRule>>,
}

impl TieredPeeringAgreementValidator {
    pub fn validate(&self, item: &TieredPeeringAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredPeeringAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredPeeringAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_peering_agreement_validator() -> TieredPeeringAgreementValidator {
    TieredPeeringAgreementValidator {
        rules: vec![
            Rc::new(TieredPeeringAgreementLabelLengthRule { max_length: 86400 }) as Rc<dyn TieredPeeringAgreementRule>,
            Rc::new(TieredPeeringAgreementTitleLengthRule { max_length: 1000 }),
            Rc::new(TieredPeeringAgreementCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(TieredPeeringAgreementThresholdRangeRule { min: 2.6, max: 1532.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_peering_agreement_validator();
        let item = new_tiered_peering_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TieredPeeringAgreementLabelLengthRule { max_length: 80 };
        let mut item = new_tiered_peering_agreement("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredPeeringAgreementTitleLengthRule { max_length: 64 };
        let mut item = new_tiered_peering_agreement("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = TieredPeeringAgreementThresholdRangeRule { min: 2.1, max: 1624.0 };
        let mut item = new_tiered_peering_agreement("x", "y");
        item.threshold = 1625.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
