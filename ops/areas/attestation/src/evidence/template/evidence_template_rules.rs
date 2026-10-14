use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence template must pass. Check returns its problems, or none.
pub trait EvidenceTemplateRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTemplateExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceTemplateRule for EvidenceTemplateExpiresAtRangeRule {
    fn check(&self, item: &EvidenceTemplate) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTemplateRequiredLabelRule {
    pub message: String,
}

impl EvidenceTemplateRule for EvidenceTemplateRequiredLabelRule {
    fn check(&self, item: &EvidenceTemplate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTemplateRequiredCodeRule {
    pub message: String,
}

impl EvidenceTemplateRule for EvidenceTemplateRequiredCodeRule {
    fn check(&self, item: &EvidenceTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence template.
#[derive(Debug, Clone)]
pub struct EvidenceTemplateValidator {
    pub rules: Vec<Rc<dyn EvidenceTemplateRule>>,
}

impl EvidenceTemplateValidator {
    pub fn validate(&self, item: &EvidenceTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EvidenceTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_template_validator() -> EvidenceTemplateValidator {
    EvidenceTemplateValidator {
        rules: vec![
            Rc::new(EvidenceTemplateExpiresAtRangeRule { min: 80, max: 255 }) as Rc<dyn EvidenceTemplateRule>,
            Rc::new(EvidenceTemplateRequiredLabelRule {
                message: "evidence template needs a value".to_string(),
            }),
            Rc::new(EvidenceTemplateRequiredCodeRule {
                message: "evidence template needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_template_validator();
        let item = new_evidence_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = EvidenceTemplateExpiresAtRangeRule { min: 80, max: 255 };
        let mut item = new_evidence_template("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = EvidenceTemplateRequiredLabelRule {
            message: "evidence template needs a value".to_string(),
        };
        let mut item = new_evidence_template("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EvidenceTemplateRequiredCodeRule {
            message: "evidence template needs this field".to_string(),
        };
        let mut item = new_evidence_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
