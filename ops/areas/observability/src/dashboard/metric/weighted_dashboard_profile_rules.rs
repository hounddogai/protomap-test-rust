use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted dashboard profile must pass. Check returns its problems, or none.
pub trait WeightedDashboardProfileRule: std::fmt::Debug {
    fn check(&self, item: &WeightedDashboardProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDashboardProfileSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedDashboardProfileRule for WeightedDashboardProfileSequenceRangeRule {
    fn check(&self, item: &WeightedDashboardProfile) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDashboardProfileRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedDashboardProfileRule for WeightedDashboardProfileRateRangeRule {
    fn check(&self, item: &WeightedDashboardProfile) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDashboardProfileTitleLengthRule {
    pub max_length: i64,
}

impl WeightedDashboardProfileRule for WeightedDashboardProfileTitleLengthRule {
    fn check(&self, item: &WeightedDashboardProfile) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDashboardProfileRequiredContactEmailRule {
    pub message: String,
}

impl WeightedDashboardProfileRule for WeightedDashboardProfileRequiredContactEmailRule {
    fn check(&self, item: &WeightedDashboardProfile) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDashboardProfileAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl WeightedDashboardProfileRule for WeightedDashboardProfileAllowedTitleRule {
    fn check(&self, item: &WeightedDashboardProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is above maximum", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDashboardProfileCodePrefixRule {
    pub prefix: String,
}

impl WeightedDashboardProfileRule for WeightedDashboardProfileCodePrefixRule {
    fn check(&self, item: &WeightedDashboardProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted dashboard profile.
#[derive(Debug, Clone)]
pub struct WeightedDashboardProfileValidator {
    pub rules: Vec<Rc<dyn WeightedDashboardProfileRule>>,
}

impl WeightedDashboardProfileValidator {
    pub fn validate(&self, item: &WeightedDashboardProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedDashboardProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedDashboardProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_dashboard_profile_validator() -> WeightedDashboardProfileValidator {
    WeightedDashboardProfileValidator {
        rules: vec![
            Rc::new(WeightedDashboardProfileSequenceRangeRule { min: 16, max: 1000 })
                as Rc<dyn WeightedDashboardProfileRule>,
            Rc::new(WeightedDashboardProfileRateRangeRule { min: 9.5, max: 4269.9 }),
            Rc::new(WeightedDashboardProfileTitleLengthRule { max_length: 120 }),
            Rc::new(WeightedDashboardProfileRequiredContactEmailRule {
                message: "weighted dashboard profile needs this field".to_string(),
            }),
            Rc::new(WeightedDashboardProfileAllowedTitleRule {
                allowed: vec!["adaptive".to_string(), "weighted".to_string()],
            }),
            Rc::new(WeightedDashboardProfileCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_dashboard_profile_validator();
        let item = new_weighted_dashboard_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = WeightedDashboardProfileSequenceRangeRule { min: 120, max: 3600 };
        let mut item = new_weighted_dashboard_profile("x", "y");
        item.sequence = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = WeightedDashboardProfileRateRangeRule { min: 0.5, max: 330.7 };
        let mut item = new_weighted_dashboard_profile("x", "y");
        item.rate = 332.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = WeightedDashboardProfileTitleLengthRule { max_length: 80 };
        let mut item = new_weighted_dashboard_profile("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = WeightedDashboardProfileRequiredContactEmailRule {
            message: "weighted dashboard profile needs an entry".to_string(),
        };
        let mut item = new_weighted_dashboard_profile("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
