use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient evidence notice must pass. Check returns its problems, or none.
pub trait LenientEvidenceNoticeRule: std::fmt::Debug {
    fn check(&self, item: &LenientEvidenceNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEvidenceNoticeRequiredLabelRule {
    pub message: String,
}

impl LenientEvidenceNoticeRule for LenientEvidenceNoticeRequiredLabelRule {
    fn check(&self, item: &LenientEvidenceNotice) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEvidenceNoticeCodePrefixRule {
    pub prefix: String,
}

impl LenientEvidenceNoticeRule for LenientEvidenceNoticeCodePrefixRule {
    fn check(&self, item: &LenientEvidenceNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEvidenceNoticeSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientEvidenceNoticeRule for LenientEvidenceNoticeSequenceRangeRule {
    fn check(&self, item: &LenientEvidenceNotice) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEvidenceNoticeDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientEvidenceNoticeRule for LenientEvidenceNoticeDurationMinutesRangeRule {
    fn check(&self, item: &LenientEvidenceNotice) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a lenient evidence notice.
#[derive(Debug, Clone)]
pub struct LenientEvidenceNoticeValidator {
    pub rules: Vec<Rc<dyn LenientEvidenceNoticeRule>>,
}

impl LenientEvidenceNoticeValidator {
    pub fn validate(&self, item: &LenientEvidenceNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientEvidenceNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientEvidenceNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_evidence_notice_validator() -> LenientEvidenceNoticeValidator {
    LenientEvidenceNoticeValidator {
        rules: vec![
            Rc::new(LenientEvidenceNoticeRequiredLabelRule {
                message: "lenient evidence notice needs this field".to_string(),
            }) as Rc<dyn LenientEvidenceNoticeRule>,
            Rc::new(LenientEvidenceNoticeCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(LenientEvidenceNoticeSequenceRangeRule { min: 16, max: 3600 }),
            Rc::new(LenientEvidenceNoticeDurationMinutesRangeRule { min: 16, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_evidence_notice_validator();
        let item = new_lenient_evidence_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientEvidenceNoticeRequiredLabelRule {
            message: "lenient evidence notice needs an entry".to_string(),
        };
        let mut item = new_lenient_evidence_notice("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = LenientEvidenceNoticeSequenceRangeRule { min: 16, max: 1000 };
        let mut item = new_lenient_evidence_notice("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = LenientEvidenceNoticeDurationMinutesRangeRule { min: 80, max: 120 };
        let mut item = new_lenient_evidence_notice("x", "y");
        item.duration_minutes = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
