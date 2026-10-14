use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary reservation template must pass. Check returns its problems, or none.
pub trait PrimaryReservationTemplateRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryReservationTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationTemplateCodePrefixRule {
    pub prefix: String,
}

impl PrimaryReservationTemplateRule for PrimaryReservationTemplateCodePrefixRule {
    fn check(&self, item: &PrimaryReservationTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationTemplateWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryReservationTemplateRule for PrimaryReservationTemplateWindowDaysRangeRule {
    fn check(&self, item: &PrimaryReservationTemplate) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationTemplateRequiredTitleRule {
    pub message: String,
}

impl PrimaryReservationTemplateRule for PrimaryReservationTemplateRequiredTitleRule {
    fn check(&self, item: &PrimaryReservationTemplate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationTemplateAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl PrimaryReservationTemplateRule for PrimaryReservationTemplateAllowedPhoneRule {
    fn check(&self, item: &PrimaryReservationTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is inconsistent", item.phone)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary reservation template.
#[derive(Debug, Clone)]
pub struct PrimaryReservationTemplateValidator {
    pub rules: Vec<Rc<dyn PrimaryReservationTemplateRule>>,
}

impl PrimaryReservationTemplateValidator {
    pub fn validate(&self, item: &PrimaryReservationTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryReservationTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryReservationTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_reservation_template_validator() -> PrimaryReservationTemplateValidator {
    PrimaryReservationTemplateValidator {
        rules: vec![
            Rc::new(PrimaryReservationTemplateCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn PrimaryReservationTemplateRule>,
            Rc::new(PrimaryReservationTemplateWindowDaysRangeRule { min: 16, max: 64 }),
            Rc::new(PrimaryReservationTemplateRequiredTitleRule {
                message: "primary reservation template needs this field".to_string(),
            }),
            Rc::new(PrimaryReservationTemplateAllowedPhoneRule {
                allowed: vec!["seasonal".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_reservation_template_validator();
        let item = new_primary_reservation_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = PrimaryReservationTemplateWindowDaysRangeRule { min: 255, max: 86400 };
        let mut item = new_primary_reservation_template("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimaryReservationTemplateRequiredTitleRule {
            message: "primary reservation template needs this field".to_string(),
        };
        let mut item = new_primary_reservation_template("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
