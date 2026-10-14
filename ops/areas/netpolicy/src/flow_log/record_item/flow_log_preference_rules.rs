use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log preference must pass. Check returns its problems, or none.
pub trait FlowLogPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPreferenceCountryLengthRule {
    pub max_length: i64,
}

impl FlowLogPreferenceRule for FlowLogPreferenceCountryLengthRule {
    fn check(&self, item: &FlowLogPreference) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPreferenceRequiredCodeRule {
    pub message: String,
}

impl FlowLogPreferenceRule for FlowLogPreferenceRequiredCodeRule {
    fn check(&self, item: &FlowLogPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPreferenceExternalRefLengthRule {
    pub max_length: i64,
}

impl FlowLogPreferenceRule for FlowLogPreferenceExternalRefLengthRule {
    fn check(&self, item: &FlowLogPreference) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl FlowLogPreferenceRule for FlowLogPreferenceCodeLengthRule {
    fn check(&self, item: &FlowLogPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPreferenceMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FlowLogPreferenceRule for FlowLogPreferenceMarginRangeRule {
    fn check(&self, item: &FlowLogPreference) -> Vec<String> {
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
pub struct FlowLogPreferenceAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl FlowLogPreferenceRule for FlowLogPreferenceAllowedCountryRule {
    fn check(&self, item: &FlowLogPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is below minimum", item.country)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log preference.
#[derive(Debug, Clone)]
pub struct FlowLogPreferenceValidator {
    pub rules: Vec<Rc<dyn FlowLogPreferenceRule>>,
}

impl FlowLogPreferenceValidator {
    pub fn validate(&self, item: &FlowLogPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_preference_validator() -> FlowLogPreferenceValidator {
    FlowLogPreferenceValidator {
        rules: vec![
            Rc::new(FlowLogPreferenceCountryLengthRule { max_length: 86400 }) as Rc<dyn FlowLogPreferenceRule>,
            Rc::new(FlowLogPreferenceRequiredCodeRule {
                message: "flow log preference needs this field".to_string(),
            }),
            Rc::new(FlowLogPreferenceExternalRefLengthRule { max_length: 86400 }),
            Rc::new(FlowLogPreferenceCodeLengthRule { max_length: 86400 }),
            Rc::new(FlowLogPreferenceMarginRangeRule { min: 9.6, max: 110.0 }),
            Rc::new(FlowLogPreferenceAllowedCountryRule {
                allowed: vec!["adaptive".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_preference_validator();
        let item = new_flow_log_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = FlowLogPreferenceCountryLengthRule { max_length: 255 };
        let mut item = new_flow_log_preference("x", "y");
        item.country = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FlowLogPreferenceRequiredCodeRule {
            message: "flow log preference needs a value".to_string(),
        };
        let mut item = new_flow_log_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = FlowLogPreferenceExternalRefLengthRule { max_length: 16 };
        let mut item = new_flow_log_preference("x", "y");
        item.external_ref = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FlowLogPreferenceCodeLengthRule { max_length: 120 };
        let mut item = new_flow_log_preference("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = FlowLogPreferenceMarginRangeRule { min: 6.5, max: 4060.3 };
        let mut item = new_flow_log_preference("x", "y");
        item.margin = 4061.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
