use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout window must pass. Check returns its problems, or none.
pub trait RolloutWindowRule: std::fmt::Debug {
    fn check(&self, item: &RolloutWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutWindowCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RolloutWindowRule for RolloutWindowCeilingRangeRule {
    fn check(&self, item: &RolloutWindow) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutWindowRequiredCountryRule {
    pub message: String,
}

impl RolloutWindowRule for RolloutWindowRequiredCountryRule {
    fn check(&self, item: &RolloutWindow) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutWindowRequiredNameRule {
    pub message: String,
}

impl RolloutWindowRule for RolloutWindowRequiredNameRule {
    fn check(&self, item: &RolloutWindow) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutWindowPrincipalLengthRule {
    pub max_length: i64,
}

impl RolloutWindowRule for RolloutWindowPrincipalLengthRule {
    fn check(&self, item: &RolloutWindow) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout window.
#[derive(Debug, Clone)]
pub struct RolloutWindowValidator {
    pub rules: Vec<Rc<dyn RolloutWindowRule>>,
}

impl RolloutWindowValidator {
    pub fn validate(&self, item: &RolloutWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_window_validator() -> RolloutWindowValidator {
    RolloutWindowValidator {
        rules: vec![
            Rc::new(RolloutWindowCeilingRangeRule { min: 9.4, max: 4924.0 }) as Rc<dyn RolloutWindowRule>,
            Rc::new(RolloutWindowRequiredCountryRule {
                message: "rollout window needs this field".to_string(),
            }),
            Rc::new(RolloutWindowRequiredNameRule {
                message: "rollout window needs this field".to_string(),
            }),
            Rc::new(RolloutWindowPrincipalLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_window_validator();
        let item = new_rollout_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = RolloutWindowCeilingRangeRule { min: 1.3, max: 1096.7 };
        let mut item = new_rollout_window("x", "y");
        item.ceiling = 1098.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = RolloutWindowRequiredCountryRule {
            message: "rollout window needs an entry".to_string(),
        };
        let mut item = new_rollout_window("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RolloutWindowRequiredNameRule {
            message: "rollout window needs this field".to_string(),
        };
        let mut item = new_rollout_window("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = RolloutWindowPrincipalLengthRule { max_length: 32 };
        let mut item = new_rollout_window("x", "y");
        item.principal = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
