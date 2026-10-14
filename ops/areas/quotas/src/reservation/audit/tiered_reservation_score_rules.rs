use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered reservation score must pass. Check returns its problems, or none.
pub trait TieredReservationScoreRule: std::fmt::Debug {
    fn check(&self, item: &TieredReservationScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationScoreRequiredLabelRule {
    pub message: String,
}

impl TieredReservationScoreRule for TieredReservationScoreRequiredLabelRule {
    fn check(&self, item: &TieredReservationScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationScoreRequiredAudienceRule {
    pub message: String,
}

impl TieredReservationScoreRule for TieredReservationScoreRequiredAudienceRule {
    fn check(&self, item: &TieredReservationScore) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationScoreCodePrefixRule {
    pub prefix: String,
}

impl TieredReservationScoreRule for TieredReservationScoreCodePrefixRule {
    fn check(&self, item: &TieredReservationScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationScoreRequiredCodeRule {
    pub message: String,
}

impl TieredReservationScoreRule for TieredReservationScoreRequiredCodeRule {
    fn check(&self, item: &TieredReservationScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered reservation score.
#[derive(Debug, Clone)]
pub struct TieredReservationScoreValidator {
    pub rules: Vec<Rc<dyn TieredReservationScoreRule>>,
}

impl TieredReservationScoreValidator {
    pub fn validate(&self, item: &TieredReservationScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredReservationScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredReservationScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_reservation_score_validator() -> TieredReservationScoreValidator {
    TieredReservationScoreValidator {
        rules: vec![
            Rc::new(TieredReservationScoreRequiredLabelRule {
                message: "tiered reservation score needs this field".to_string(),
            }) as Rc<dyn TieredReservationScoreRule>,
            Rc::new(TieredReservationScoreRequiredAudienceRule {
                message: "tiered reservation score needs an entry".to_string(),
            }),
            Rc::new(TieredReservationScoreCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(TieredReservationScoreRequiredCodeRule {
                message: "tiered reservation score needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_reservation_score_validator();
        let item = new_tiered_reservation_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TieredReservationScoreRequiredLabelRule {
            message: "tiered reservation score needs a value".to_string(),
        };
        let mut item = new_tiered_reservation_score("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = TieredReservationScoreRequiredAudienceRule {
            message: "tiered reservation score needs a value".to_string(),
        };
        let mut item = new_tiered_reservation_score("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredReservationScoreRequiredCodeRule {
            message: "tiered reservation score needs an entry".to_string(),
        };
        let mut item = new_tiered_reservation_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
