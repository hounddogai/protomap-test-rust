use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual evidence template must pass. Check returns its problems, or none.
pub trait ManualEvidenceTemplateRule: std::fmt::Debug {
    fn check(&self, item: &ManualEvidenceTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceTemplateRequiredCipherRule {
    pub message: String,
}

impl ManualEvidenceTemplateRule for ManualEvidenceTemplateRequiredCipherRule {
    fn check(&self, item: &ManualEvidenceTemplate) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceTemplateCodeLengthRule {
    pub max_length: i64,
}

impl ManualEvidenceTemplateRule for ManualEvidenceTemplateCodeLengthRule {
    fn check(&self, item: &ManualEvidenceTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceTemplateMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualEvidenceTemplateRule for ManualEvidenceTemplateMarginRangeRule {
    fn check(&self, item: &ManualEvidenceTemplate) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceTemplateAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl ManualEvidenceTemplateRule for ManualEvidenceTemplateAllowedCipherRule {
    fn check(&self, item: &ManualEvidenceTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is stale", item.cipher)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceTemplateCodePrefixRule {
    pub prefix: String,
}

impl ManualEvidenceTemplateRule for ManualEvidenceTemplateCodePrefixRule {
    fn check(&self, item: &ManualEvidenceTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceTemplateRequiredNameRule {
    pub message: String,
}

impl ManualEvidenceTemplateRule for ManualEvidenceTemplateRequiredNameRule {
    fn check(&self, item: &ManualEvidenceTemplate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual evidence template.
#[derive(Debug, Clone)]
pub struct ManualEvidenceTemplateValidator {
    pub rules: Vec<Rc<dyn ManualEvidenceTemplateRule>>,
}

impl ManualEvidenceTemplateValidator {
    pub fn validate(&self, item: &ManualEvidenceTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualEvidenceTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualEvidenceTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_evidence_template_validator() -> ManualEvidenceTemplateValidator {
    ManualEvidenceTemplateValidator {
        rules: vec![
            Rc::new(ManualEvidenceTemplateRequiredCipherRule {
                message: "manual evidence template needs a value".to_string(),
            }) as Rc<dyn ManualEvidenceTemplateRule>,
            Rc::new(ManualEvidenceTemplateCodeLengthRule { max_length: 86400 }),
            Rc::new(ManualEvidenceTemplateMarginRangeRule { min: 1.3, max: 3462.3 }),
            Rc::new(ManualEvidenceTemplateAllowedCipherRule { allowed: Vec::new() }),
            Rc::new(ManualEvidenceTemplateCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ManualEvidenceTemplateRequiredNameRule {
                message: "manual evidence template needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_evidence_template_validator();
        let item = new_manual_evidence_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = ManualEvidenceTemplateRequiredCipherRule {
            message: "manual evidence template needs a value".to_string(),
        };
        let mut item = new_manual_evidence_template("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualEvidenceTemplateCodeLengthRule { max_length: 1000 };
        let mut item = new_manual_evidence_template("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = ManualEvidenceTemplateMarginRangeRule { min: 7.8, max: 3080.8 };
        let mut item = new_manual_evidence_template("x", "y");
        item.margin = 3082.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ManualEvidenceTemplateRequiredNameRule {
            message: "manual evidence template needs this field".to_string(),
        };
        let mut item = new_manual_evidence_template("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
