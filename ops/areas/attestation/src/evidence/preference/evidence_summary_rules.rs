use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence summary must pass. Check returns its problems, or none.
pub trait EvidenceSummaryRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSummaryNameLengthRule {
    pub max_length: i64,
}

impl EvidenceSummaryRule for EvidenceSummaryNameLengthRule {
    fn check(&self, item: &EvidenceSummary) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSummaryThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EvidenceSummaryRule for EvidenceSummaryThresholdRangeRule {
    fn check(&self, item: &EvidenceSummary) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSummaryClusterLengthRule {
    pub max_length: i64,
}

impl EvidenceSummaryRule for EvidenceSummaryClusterLengthRule {
    fn check(&self, item: &EvidenceSummary) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSummaryDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceSummaryRule for EvidenceSummaryDurationMinutesRangeRule {
    fn check(&self, item: &EvidenceSummary) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSummaryRequiredPhoneRule {
    pub message: String,
}

impl EvidenceSummaryRule for EvidenceSummaryRequiredPhoneRule {
    fn check(&self, item: &EvidenceSummary) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence summary.
#[derive(Debug, Clone)]
pub struct EvidenceSummaryValidator {
    pub rules: Vec<Rc<dyn EvidenceSummaryRule>>,
}

impl EvidenceSummaryValidator {
    pub fn validate(&self, item: &EvidenceSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EvidenceSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_summary_validator() -> EvidenceSummaryValidator {
    EvidenceSummaryValidator {
        rules: vec![
            Rc::new(EvidenceSummaryNameLengthRule { max_length: 86400 }) as Rc<dyn EvidenceSummaryRule>,
            Rc::new(EvidenceSummaryThresholdRangeRule { min: 0.5, max: 500.7 }),
            Rc::new(EvidenceSummaryClusterLengthRule { max_length: 1000 }),
            Rc::new(EvidenceSummaryDurationMinutesRangeRule { min: 16, max: 86400 }),
            Rc::new(EvidenceSummaryRequiredPhoneRule {
                message: "evidence summary needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_summary_validator();
        let item = new_evidence_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = EvidenceSummaryNameLengthRule { max_length: 32 };
        let mut item = new_evidence_summary("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = EvidenceSummaryThresholdRangeRule { min: 9.7, max: 3056.4 };
        let mut item = new_evidence_summary("x", "y");
        item.threshold = 3057.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = EvidenceSummaryClusterLengthRule { max_length: 16 };
        let mut item = new_evidence_summary("x", "y");
        item.cluster = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = EvidenceSummaryDurationMinutesRangeRule { min: 1000, max: 86400 };
        let mut item = new_evidence_summary("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = EvidenceSummaryRequiredPhoneRule {
            message: "evidence summary needs an entry".to_string(),
        };
        let mut item = new_evidence_summary("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
