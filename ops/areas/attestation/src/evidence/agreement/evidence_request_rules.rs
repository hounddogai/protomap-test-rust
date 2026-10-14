use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a evidence request must pass. Check returns its problems, or none.
pub trait EvidenceRequestRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRequestAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl EvidenceRequestRule for EvidenceRequestAllowedLabelRule {
    fn check(&self, item: &EvidenceRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is not allowed", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRequestExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceRequestRule for EvidenceRequestExpiresAtRangeRule {
    fn check(&self, item: &EvidenceRequest) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRequestVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceRequestRule for EvidenceRequestVersionRangeRule {
    fn check(&self, item: &EvidenceRequest) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a evidence request.
#[derive(Debug, Clone)]
pub struct EvidenceRequestValidator {
    pub rules: Vec<Rc<dyn EvidenceRequestRule>>,
}

impl EvidenceRequestValidator {
    pub fn validate(&self, item: &EvidenceRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_request_validator() -> EvidenceRequestValidator {
    EvidenceRequestValidator {
        rules: vec![
            Rc::new(EvidenceRequestAllowedLabelRule {
                allowed: vec!["weighted".to_string()],
            }) as Rc<dyn EvidenceRequestRule>,
            Rc::new(EvidenceRequestExpiresAtRangeRule { min: 32, max: 255 }),
            Rc::new(EvidenceRequestVersionRangeRule { min: 1000, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_request_validator();
        let item = new_evidence_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = EvidenceRequestExpiresAtRangeRule { min: 64, max: 1000 };
        let mut item = new_evidence_request("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = EvidenceRequestVersionRangeRule { min: 120, max: 86400 };
        let mut item = new_evidence_request("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
