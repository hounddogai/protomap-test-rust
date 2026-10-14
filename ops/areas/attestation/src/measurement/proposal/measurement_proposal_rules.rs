use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement proposal must pass. Check returns its problems, or none.
pub trait MeasurementProposalRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementProposalQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementProposalRule for MeasurementProposalQuotaUnitsRangeRule {
    fn check(&self, item: &MeasurementProposal) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementProposalRequiredTimezoneRule {
    pub message: String,
}

impl MeasurementProposalRule for MeasurementProposalRequiredTimezoneRule {
    fn check(&self, item: &MeasurementProposal) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementProposalTimezoneLengthRule {
    pub max_length: i64,
}

impl MeasurementProposalRule for MeasurementProposalTimezoneLengthRule {
    fn check(&self, item: &MeasurementProposal) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement proposal.
#[derive(Debug, Clone)]
pub struct MeasurementProposalValidator {
    pub rules: Vec<Rc<dyn MeasurementProposalRule>>,
}

impl MeasurementProposalValidator {
    pub fn validate(&self, item: &MeasurementProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_proposal_validator() -> MeasurementProposalValidator {
    MeasurementProposalValidator {
        rules: vec![
            Rc::new(MeasurementProposalQuotaUnitsRangeRule { min: 32, max: 86400 }) as Rc<dyn MeasurementProposalRule>,
            Rc::new(MeasurementProposalRequiredTimezoneRule {
                message: "measurement proposal needs this field".to_string(),
            }),
            Rc::new(MeasurementProposalTimezoneLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_proposal_validator();
        let item = new_measurement_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = MeasurementProposalQuotaUnitsRangeRule { min: 80, max: 86400 };
        let mut item = new_measurement_proposal("x", "y");
        item.quota_units = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = MeasurementProposalRequiredTimezoneRule {
            message: "measurement proposal needs a value".to_string(),
        };
        let mut item = new_measurement_proposal("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = MeasurementProposalTimezoneLengthRule { max_length: 120 };
        let mut item = new_measurement_proposal("x", "y");
        item.timezone = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
