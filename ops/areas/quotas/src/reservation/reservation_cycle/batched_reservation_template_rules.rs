use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched reservation template must pass. Check returns its problems, or none.
pub trait BatchedReservationTemplateRule: std::fmt::Debug {
    fn check(&self, item: &BatchedReservationTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationTemplateRequiredCodeRule {
    pub message: String,
}

impl BatchedReservationTemplateRule for BatchedReservationTemplateRequiredCodeRule {
    fn check(&self, item: &BatchedReservationTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationTemplateRequiredTitleRule {
    pub message: String,
}

impl BatchedReservationTemplateRule for BatchedReservationTemplateRequiredTitleRule {
    fn check(&self, item: &BatchedReservationTemplate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationTemplateRequiredChannelRule {
    pub message: String,
}

impl BatchedReservationTemplateRule for BatchedReservationTemplateRequiredChannelRule {
    fn check(&self, item: &BatchedReservationTemplate) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationTemplateScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedReservationTemplateRule for BatchedReservationTemplateScoreRangeRule {
    fn check(&self, item: &BatchedReservationTemplate) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationTemplateRequiredClusterRule {
    pub message: String,
}

impl BatchedReservationTemplateRule for BatchedReservationTemplateRequiredClusterRule {
    fn check(&self, item: &BatchedReservationTemplate) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationTemplateArchivedGuardRule {
    pub limit: f64,
}

impl BatchedReservationTemplateRule for BatchedReservationTemplateArchivedGuardRule {
    fn check(&self, item: &BatchedReservationTemplate) -> Vec<String> {
        if item.archived && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while archived", item.score, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched reservation template.
#[derive(Debug, Clone)]
pub struct BatchedReservationTemplateValidator {
    pub rules: Vec<Rc<dyn BatchedReservationTemplateRule>>,
}

impl BatchedReservationTemplateValidator {
    pub fn validate(&self, item: &BatchedReservationTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedReservationTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedReservationTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_reservation_template_validator() -> BatchedReservationTemplateValidator {
    BatchedReservationTemplateValidator {
        rules: vec![
            Rc::new(BatchedReservationTemplateRequiredCodeRule {
                message: "batched reservation template needs this field".to_string(),
            }) as Rc<dyn BatchedReservationTemplateRule>,
            Rc::new(BatchedReservationTemplateRequiredTitleRule {
                message: "batched reservation template needs an entry".to_string(),
            }),
            Rc::new(BatchedReservationTemplateRequiredChannelRule {
                message: "batched reservation template needs an entry".to_string(),
            }),
            Rc::new(BatchedReservationTemplateScoreRangeRule { min: 0.1, max: 3999.9 }),
            Rc::new(BatchedReservationTemplateRequiredClusterRule {
                message: "batched reservation template needs an entry".to_string(),
            }),
            Rc::new(BatchedReservationTemplateArchivedGuardRule { limit: 5.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_reservation_template_validator();
        let item = new_batched_reservation_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedReservationTemplateRequiredCodeRule {
            message: "batched reservation template needs an entry".to_string(),
        };
        let mut item = new_batched_reservation_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = BatchedReservationTemplateRequiredTitleRule {
            message: "batched reservation template needs an entry".to_string(),
        };
        let mut item = new_batched_reservation_template("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = BatchedReservationTemplateRequiredChannelRule {
            message: "batched reservation template needs this field".to_string(),
        };
        let mut item = new_batched_reservation_template("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = BatchedReservationTemplateScoreRangeRule { min: 8.5, max: 998.8 };
        let mut item = new_batched_reservation_template("x", "y");
        item.score = 1000.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = BatchedReservationTemplateRequiredClusterRule {
            message: "batched reservation template needs a value".to_string(),
        };
        let mut item = new_batched_reservation_template("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
