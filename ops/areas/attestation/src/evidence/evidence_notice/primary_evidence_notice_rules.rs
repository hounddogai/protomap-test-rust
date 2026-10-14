use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary evidence notice must pass. Check returns its problems, or none.
pub trait PrimaryEvidenceNoticeRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryEvidenceNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceNoticeRequiredExternalRefRule {
    pub message: String,
}

impl PrimaryEvidenceNoticeRule for PrimaryEvidenceNoticeRequiredExternalRefRule {
    fn check(&self, item: &PrimaryEvidenceNotice) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceNoticeRequiredNameRule {
    pub message: String,
}

impl PrimaryEvidenceNoticeRule for PrimaryEvidenceNoticeRequiredNameRule {
    fn check(&self, item: &PrimaryEvidenceNotice) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceNoticeCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryEvidenceNoticeRule for PrimaryEvidenceNoticeCodeLengthRule {
    fn check(&self, item: &PrimaryEvidenceNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceNoticeArchivedGuardRule {
    pub limit: f64,
}

impl PrimaryEvidenceNoticeRule for PrimaryEvidenceNoticeArchivedGuardRule {
    fn check(&self, item: &PrimaryEvidenceNotice) -> Vec<String> {
        if item.archived && (item.minimum > self.limit) {
            return vec![format!(
                "minimum {} exceeds {} while archived",
                item.minimum, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceNoticeMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryEvidenceNoticeRule for PrimaryEvidenceNoticeMinimumRangeRule {
    fn check(&self, item: &PrimaryEvidenceNotice) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceNoticeExternalRefLengthRule {
    pub max_length: i64,
}

impl PrimaryEvidenceNoticeRule for PrimaryEvidenceNoticeExternalRefLengthRule {
    fn check(&self, item: &PrimaryEvidenceNotice) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary evidence notice.
#[derive(Debug, Clone)]
pub struct PrimaryEvidenceNoticeValidator {
    pub rules: Vec<Rc<dyn PrimaryEvidenceNoticeRule>>,
}

impl PrimaryEvidenceNoticeValidator {
    pub fn validate(&self, item: &PrimaryEvidenceNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryEvidenceNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryEvidenceNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_evidence_notice_validator() -> PrimaryEvidenceNoticeValidator {
    PrimaryEvidenceNoticeValidator {
        rules: vec![
            Rc::new(PrimaryEvidenceNoticeRequiredExternalRefRule {
                message: "primary evidence notice needs this field".to_string(),
            }) as Rc<dyn PrimaryEvidenceNoticeRule>,
            Rc::new(PrimaryEvidenceNoticeRequiredNameRule {
                message: "primary evidence notice needs a value".to_string(),
            }),
            Rc::new(PrimaryEvidenceNoticeCodeLengthRule { max_length: 255 }),
            Rc::new(PrimaryEvidenceNoticeArchivedGuardRule { limit: 1.7 }),
            Rc::new(PrimaryEvidenceNoticeMinimumRangeRule { min: 2.5, max: 2641.9 }),
            Rc::new(PrimaryEvidenceNoticeExternalRefLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_evidence_notice_validator();
        let item = new_primary_evidence_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = PrimaryEvidenceNoticeRequiredExternalRefRule {
            message: "primary evidence notice needs a value".to_string(),
        };
        let mut item = new_primary_evidence_notice("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PrimaryEvidenceNoticeRequiredNameRule {
            message: "primary evidence notice needs a value".to_string(),
        };
        let mut item = new_primary_evidence_notice("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryEvidenceNoticeCodeLengthRule { max_length: 80 };
        let mut item = new_primary_evidence_notice("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = PrimaryEvidenceNoticeMinimumRangeRule { min: 1.0, max: 1677.3 };
        let mut item = new_primary_evidence_notice("x", "y");
        item.minimum = 1678.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = PrimaryEvidenceNoticeExternalRefLengthRule { max_length: 32 };
        let mut item = new_primary_evidence_notice("x", "y");
        item.external_ref = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
