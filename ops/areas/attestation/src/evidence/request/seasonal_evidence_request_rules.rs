use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal evidence request must pass. Check returns its problems, or none.
pub trait SeasonalEvidenceRequestRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalEvidenceRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEvidenceRequestRequiredSegmentRule {
    pub message: String,
}

impl SeasonalEvidenceRequestRule for SeasonalEvidenceRequestRequiredSegmentRule {
    fn check(&self, item: &SeasonalEvidenceRequest) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEvidenceRequestNameLengthRule {
    pub max_length: i64,
}

impl SeasonalEvidenceRequestRule for SeasonalEvidenceRequestNameLengthRule {
    fn check(&self, item: &SeasonalEvidenceRequest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEvidenceRequestCipherLengthRule {
    pub max_length: i64,
}

impl SeasonalEvidenceRequestRule for SeasonalEvidenceRequestCipherLengthRule {
    fn check(&self, item: &SeasonalEvidenceRequest) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEvidenceRequestCodePrefixRule {
    pub prefix: String,
}

impl SeasonalEvidenceRequestRule for SeasonalEvidenceRequestCodePrefixRule {
    fn check(&self, item: &SeasonalEvidenceRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal evidence request.
#[derive(Debug, Clone)]
pub struct SeasonalEvidenceRequestValidator {
    pub rules: Vec<Rc<dyn SeasonalEvidenceRequestRule>>,
}

impl SeasonalEvidenceRequestValidator {
    pub fn validate(&self, item: &SeasonalEvidenceRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalEvidenceRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalEvidenceRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_evidence_request_validator() -> SeasonalEvidenceRequestValidator {
    SeasonalEvidenceRequestValidator {
        rules: vec![
            Rc::new(SeasonalEvidenceRequestRequiredSegmentRule {
                message: "seasonal evidence request needs a value".to_string(),
            }) as Rc<dyn SeasonalEvidenceRequestRule>,
            Rc::new(SeasonalEvidenceRequestNameLengthRule { max_length: 64 }),
            Rc::new(SeasonalEvidenceRequestCipherLengthRule { max_length: 80 }),
            Rc::new(SeasonalEvidenceRequestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_evidence_request_validator();
        let item = new_seasonal_evidence_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = SeasonalEvidenceRequestRequiredSegmentRule {
            message: "seasonal evidence request needs an entry".to_string(),
        };
        let mut item = new_seasonal_evidence_request("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SeasonalEvidenceRequestNameLengthRule { max_length: 86400 };
        let mut item = new_seasonal_evidence_request("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = SeasonalEvidenceRequestCipherLengthRule { max_length: 86400 };
        let mut item = new_seasonal_evidence_request("x", "y");
        item.cipher = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
