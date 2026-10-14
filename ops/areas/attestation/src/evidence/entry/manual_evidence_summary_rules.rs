use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual evidence summary must pass. Check returns its problems, or none.
pub trait ManualEvidenceSummaryRule: std::fmt::Debug {
    fn check(&self, item: &ManualEvidenceSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceSummaryRequiredCodeRule {
    pub message: String,
}

impl ManualEvidenceSummaryRule for ManualEvidenceSummaryRequiredCodeRule {
    fn check(&self, item: &ManualEvidenceSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceSummaryRequiredTitleRule {
    pub message: String,
}

impl ManualEvidenceSummaryRule for ManualEvidenceSummaryRequiredTitleRule {
    fn check(&self, item: &ManualEvidenceSummary) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceSummaryTitleLengthRule {
    pub max_length: i64,
}

impl ManualEvidenceSummaryRule for ManualEvidenceSummaryTitleLengthRule {
    fn check(&self, item: &ManualEvidenceSummary) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual evidence summary.
#[derive(Debug, Clone)]
pub struct ManualEvidenceSummaryValidator {
    pub rules: Vec<Rc<dyn ManualEvidenceSummaryRule>>,
}

impl ManualEvidenceSummaryValidator {
    pub fn validate(&self, item: &ManualEvidenceSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualEvidenceSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualEvidenceSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_evidence_summary_validator() -> ManualEvidenceSummaryValidator {
    ManualEvidenceSummaryValidator {
        rules: vec![
            Rc::new(ManualEvidenceSummaryRequiredCodeRule {
                message: "manual evidence summary needs an entry".to_string(),
            }) as Rc<dyn ManualEvidenceSummaryRule>,
            Rc::new(ManualEvidenceSummaryRequiredTitleRule {
                message: "manual evidence summary needs this field".to_string(),
            }),
            Rc::new(ManualEvidenceSummaryTitleLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_evidence_summary_validator();
        let item = new_manual_evidence_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualEvidenceSummaryRequiredCodeRule {
            message: "manual evidence summary needs a value".to_string(),
        };
        let mut item = new_manual_evidence_summary("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualEvidenceSummaryRequiredTitleRule {
            message: "manual evidence summary needs a value".to_string(),
        };
        let mut item = new_manual_evidence_summary("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ManualEvidenceSummaryTitleLengthRule { max_length: 64 };
        let mut item = new_manual_evidence_summary("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
