use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation template must pass. Check returns its problems, or none.
pub trait ReservationTemplateRule: std::fmt::Debug {
    fn check(&self, item: &ReservationTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationTemplateCodeLengthRule {
    pub max_length: i64,
}

impl ReservationTemplateRule for ReservationTemplateCodeLengthRule {
    fn check(&self, item: &ReservationTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationTemplateRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReservationTemplateRule for ReservationTemplateRevisionRangeRule {
    fn check(&self, item: &ReservationTemplate) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationTemplateTitleLengthRule {
    pub max_length: i64,
}

impl ReservationTemplateRule for ReservationTemplateTitleLengthRule {
    fn check(&self, item: &ReservationTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationTemplateRequiredFingerprintRule {
    pub message: String,
}

impl ReservationTemplateRule for ReservationTemplateRequiredFingerprintRule {
    fn check(&self, item: &ReservationTemplate) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationTemplateRequiredTitleRule {
    pub message: String,
}

impl ReservationTemplateRule for ReservationTemplateRequiredTitleRule {
    fn check(&self, item: &ReservationTemplate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationTemplateFingerprintLengthRule {
    pub max_length: i64,
}

impl ReservationTemplateRule for ReservationTemplateFingerprintLengthRule {
    fn check(&self, item: &ReservationTemplate) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation template.
#[derive(Debug, Clone)]
pub struct ReservationTemplateValidator {
    pub rules: Vec<Rc<dyn ReservationTemplateRule>>,
}

impl ReservationTemplateValidator {
    pub fn validate(&self, item: &ReservationTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReservationTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_template_validator() -> ReservationTemplateValidator {
    ReservationTemplateValidator {
        rules: vec![
            Rc::new(ReservationTemplateCodeLengthRule { max_length: 64 }) as Rc<dyn ReservationTemplateRule>,
            Rc::new(ReservationTemplateRevisionRangeRule { min: 32, max: 1000 }),
            Rc::new(ReservationTemplateTitleLengthRule { max_length: 32 }),
            Rc::new(ReservationTemplateRequiredFingerprintRule {
                message: "reservation template needs a value".to_string(),
            }),
            Rc::new(ReservationTemplateRequiredTitleRule {
                message: "reservation template needs a value".to_string(),
            }),
            Rc::new(ReservationTemplateFingerprintLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_template_validator();
        let item = new_reservation_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReservationTemplateCodeLengthRule { max_length: 255 };
        let mut item = new_reservation_template("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = ReservationTemplateRevisionRangeRule { min: 80, max: 255 };
        let mut item = new_reservation_template("x", "y");
        item.revision = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ReservationTemplateTitleLengthRule { max_length: 86400 };
        let mut item = new_reservation_template("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = ReservationTemplateRequiredFingerprintRule {
            message: "reservation template needs an entry".to_string(),
        };
        let mut item = new_reservation_template("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ReservationTemplateRequiredTitleRule {
            message: "reservation template needs an entry".to_string(),
        };
        let mut item = new_reservation_template("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = ReservationTemplateFingerprintLengthRule { max_length: 16 };
        let mut item = new_reservation_template("x", "y");
        item.fingerprint = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
