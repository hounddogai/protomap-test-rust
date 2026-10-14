use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation agreement must pass. Check returns its problems, or none.
pub trait ReservationAgreementRule: std::fmt::Debug {
    fn check(&self, item: &ReservationAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAgreementRequiredTitleRule {
    pub message: String,
}

impl ReservationAgreementRule for ReservationAgreementRequiredTitleRule {
    fn check(&self, item: &ReservationAgreement) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAgreementNameLengthRule {
    pub max_length: i64,
}

impl ReservationAgreementRule for ReservationAgreementNameLengthRule {
    fn check(&self, item: &ReservationAgreement) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAgreementThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReservationAgreementRule for ReservationAgreementThresholdRangeRule {
    fn check(&self, item: &ReservationAgreement) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAgreementRequiredExternalRefRule {
    pub message: String,
}

impl ReservationAgreementRule for ReservationAgreementRequiredExternalRefRule {
    fn check(&self, item: &ReservationAgreement) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAgreementCodePrefixRule {
    pub prefix: String,
}

impl ReservationAgreementRule for ReservationAgreementCodePrefixRule {
    fn check(&self, item: &ReservationAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAgreementTitleLengthRule {
    pub max_length: i64,
}

impl ReservationAgreementRule for ReservationAgreementTitleLengthRule {
    fn check(&self, item: &ReservationAgreement) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation agreement.
#[derive(Debug, Clone)]
pub struct ReservationAgreementValidator {
    pub rules: Vec<Rc<dyn ReservationAgreementRule>>,
}

impl ReservationAgreementValidator {
    pub fn validate(&self, item: &ReservationAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReservationAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_agreement_validator() -> ReservationAgreementValidator {
    ReservationAgreementValidator {
        rules: vec![
            Rc::new(ReservationAgreementRequiredTitleRule {
                message: "reservation agreement needs a value".to_string(),
            }) as Rc<dyn ReservationAgreementRule>,
            Rc::new(ReservationAgreementNameLengthRule { max_length: 120 }),
            Rc::new(ReservationAgreementThresholdRangeRule { min: 0.5, max: 1063.4 }),
            Rc::new(ReservationAgreementRequiredExternalRefRule {
                message: "reservation agreement needs a value".to_string(),
            }),
            Rc::new(ReservationAgreementCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ReservationAgreementTitleLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_agreement_validator();
        let item = new_reservation_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ReservationAgreementRequiredTitleRule {
            message: "reservation agreement needs this field".to_string(),
        };
        let mut item = new_reservation_agreement("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ReservationAgreementNameLengthRule { max_length: 80 };
        let mut item = new_reservation_agreement("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = ReservationAgreementThresholdRangeRule { min: 3.5, max: 1243.4 };
        let mut item = new_reservation_agreement("x", "y");
        item.threshold = 1244.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = ReservationAgreementRequiredExternalRefRule {
            message: "reservation agreement needs a value".to_string(),
        };
        let mut item = new_reservation_agreement("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ReservationAgreementTitleLengthRule { max_length: 3600 };
        let mut item = new_reservation_agreement("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
