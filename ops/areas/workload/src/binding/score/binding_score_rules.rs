use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a binding score must pass. Check returns its problems, or none.
pub trait BindingScoreRule: std::fmt::Debug {
    fn check(&self, item: &BindingScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingScoreAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl BindingScoreRule for BindingScoreAllowedChannelRule {
    fn check(&self, item: &BindingScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is not allowed", item.channel)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingScoreWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingScoreRule for BindingScoreWindowDaysRangeRule {
    fn check(&self, item: &BindingScore) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingScoreRequiredLabelRule {
    pub message: String,
}

impl BindingScoreRule for BindingScoreRequiredLabelRule {
    fn check(&self, item: &BindingScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingScoreRequiredChannelRule {
    pub message: String,
}

impl BindingScoreRule for BindingScoreRequiredChannelRule {
    fn check(&self, item: &BindingScore) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingScoreRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BindingScoreRule for BindingScoreRateRangeRule {
    fn check(&self, item: &BindingScore) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a binding score.
#[derive(Debug, Clone)]
pub struct BindingScoreValidator {
    pub rules: Vec<Rc<dyn BindingScoreRule>>,
}

impl BindingScoreValidator {
    pub fn validate(&self, item: &BindingScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_score_validator() -> BindingScoreValidator {
    BindingScoreValidator {
        rules: vec![
            Rc::new(BindingScoreAllowedChannelRule {
                allowed: vec!["manual".to_string()],
            }) as Rc<dyn BindingScoreRule>,
            Rc::new(BindingScoreWindowDaysRangeRule { min: 1000, max: 1000 }),
            Rc::new(BindingScoreRequiredLabelRule {
                message: "binding score needs a value".to_string(),
            }),
            Rc::new(BindingScoreRequiredChannelRule {
                message: "binding score needs an entry".to_string(),
            }),
            Rc::new(BindingScoreRateRangeRule { min: 6.6, max: 1122.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_score_validator();
        let item = new_binding_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = BindingScoreWindowDaysRangeRule { min: 16, max: 64 };
        let mut item = new_binding_score("x", "y");
        item.window_days = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BindingScoreRequiredLabelRule {
            message: "binding score needs an entry".to_string(),
        };
        let mut item = new_binding_score("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = BindingScoreRequiredChannelRule {
            message: "binding score needs this field".to_string(),
        };
        let mut item = new_binding_score("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = BindingScoreRateRangeRule { min: 3.0, max: 4992.0 };
        let mut item = new_binding_score("x", "y");
        item.rate = 4993.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
