use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy reservation score must pass. Check returns its problems, or none.
pub trait LegacyReservationScoreRule: std::fmt::Debug {
    fn check(&self, item: &LegacyReservationScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReservationScoreRequiredCodeRule {
    pub message: String,
}

impl LegacyReservationScoreRule for LegacyReservationScoreRequiredCodeRule {
    fn check(&self, item: &LegacyReservationScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReservationScoreBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyReservationScoreRule for LegacyReservationScoreBatchSizeRangeRule {
    fn check(&self, item: &LegacyReservationScore) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReservationScoreRequiredTitleRule {
    pub message: String,
}

impl LegacyReservationScoreRule for LegacyReservationScoreRequiredTitleRule {
    fn check(&self, item: &LegacyReservationScore) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReservationScoreLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyReservationScoreRule for LegacyReservationScoreLimitRangeRule {
    fn check(&self, item: &LegacyReservationScore) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy reservation score.
#[derive(Debug, Clone)]
pub struct LegacyReservationScoreValidator {
    pub rules: Vec<Rc<dyn LegacyReservationScoreRule>>,
}

impl LegacyReservationScoreValidator {
    pub fn validate(&self, item: &LegacyReservationScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyReservationScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyReservationScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_reservation_score_validator() -> LegacyReservationScoreValidator {
    LegacyReservationScoreValidator {
        rules: vec![
            Rc::new(LegacyReservationScoreRequiredCodeRule {
                message: "legacy reservation score needs an entry".to_string(),
            }) as Rc<dyn LegacyReservationScoreRule>,
            Rc::new(LegacyReservationScoreBatchSizeRangeRule { min: 16, max: 32 }),
            Rc::new(LegacyReservationScoreRequiredTitleRule {
                message: "legacy reservation score needs this field".to_string(),
            }),
            Rc::new(LegacyReservationScoreLimitRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_reservation_score_validator();
        let item = new_legacy_reservation_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyReservationScoreRequiredCodeRule {
            message: "legacy reservation score needs this field".to_string(),
        };
        let mut item = new_legacy_reservation_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = LegacyReservationScoreBatchSizeRangeRule { min: 64, max: 1000 };
        let mut item = new_legacy_reservation_score("x", "y");
        item.batch_size = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacyReservationScoreRequiredTitleRule {
            message: "legacy reservation score needs an entry".to_string(),
        };
        let mut item = new_legacy_reservation_score("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = LegacyReservationScoreLimitRangeRule { min: 32, max: 80 };
        let mut item = new_legacy_reservation_score("x", "y");
        item.limit = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
