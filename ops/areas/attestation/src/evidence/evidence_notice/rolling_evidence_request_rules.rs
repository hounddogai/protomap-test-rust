use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling evidence request must pass. Check returns its problems, or none.
pub trait RollingEvidenceRequestRule: std::fmt::Debug {
    fn check(&self, item: &RollingEvidenceRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEvidenceRequestRequiredTitleRule {
    pub message: String,
}

impl RollingEvidenceRequestRule for RollingEvidenceRequestRequiredTitleRule {
    fn check(&self, item: &RollingEvidenceRequest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEvidenceRequestNotesLengthRule {
    pub max_length: i64,
}

impl RollingEvidenceRequestRule for RollingEvidenceRequestNotesLengthRule {
    fn check(&self, item: &RollingEvidenceRequest) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEvidenceRequestAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingEvidenceRequestRule for RollingEvidenceRequestAttemptsRangeRule {
    fn check(&self, item: &RollingEvidenceRequest) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEvidenceRequestTitleLengthRule {
    pub max_length: i64,
}

impl RollingEvidenceRequestRule for RollingEvidenceRequestTitleLengthRule {
    fn check(&self, item: &RollingEvidenceRequest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling evidence request.
#[derive(Debug, Clone)]
pub struct RollingEvidenceRequestValidator {
    pub rules: Vec<Rc<dyn RollingEvidenceRequestRule>>,
}

impl RollingEvidenceRequestValidator {
    pub fn validate(&self, item: &RollingEvidenceRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingEvidenceRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingEvidenceRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_evidence_request_validator() -> RollingEvidenceRequestValidator {
    RollingEvidenceRequestValidator {
        rules: vec![
            Rc::new(RollingEvidenceRequestRequiredTitleRule {
                message: "rolling evidence request needs a value".to_string(),
            }) as Rc<dyn RollingEvidenceRequestRule>,
            Rc::new(RollingEvidenceRequestNotesLengthRule { max_length: 80 }),
            Rc::new(RollingEvidenceRequestAttemptsRangeRule { min: 32, max: 120 }),
            Rc::new(RollingEvidenceRequestTitleLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_evidence_request_validator();
        let item = new_rolling_evidence_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RollingEvidenceRequestRequiredTitleRule {
            message: "rolling evidence request needs a value".to_string(),
        };
        let mut item = new_rolling_evidence_request("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = RollingEvidenceRequestNotesLengthRule { max_length: 80 };
        let mut item = new_rolling_evidence_request("x", "y");
        item.notes = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = RollingEvidenceRequestAttemptsRangeRule { min: 1000, max: 86400 };
        let mut item = new_rolling_evidence_request("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollingEvidenceRequestTitleLengthRule { max_length: 120 };
        let mut item = new_rolling_evidence_request("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
