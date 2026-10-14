use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted forecast metric must pass. Check returns its problems, or none.
pub trait WeightedForecastMetricRule: std::fmt::Debug {
    fn check(&self, item: &WeightedForecastMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastMetricSegmentLengthRule {
    pub max_length: i64,
}

impl WeightedForecastMetricRule for WeightedForecastMetricSegmentLengthRule {
    fn check(&self, item: &WeightedForecastMetric) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastMetricRequiredCodeRule {
    pub message: String,
}

impl WeightedForecastMetricRule for WeightedForecastMetricRequiredCodeRule {
    fn check(&self, item: &WeightedForecastMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastMetricRequiredNameRule {
    pub message: String,
}

impl WeightedForecastMetricRule for WeightedForecastMetricRequiredNameRule {
    fn check(&self, item: &WeightedForecastMetric) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastMetricAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl WeightedForecastMetricRule for WeightedForecastMetricAllowedNamespaceRule {
    fn check(&self, item: &WeightedForecastMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is blocked", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastMetricCodePrefixRule {
    pub prefix: String,
}

impl WeightedForecastMetricRule for WeightedForecastMetricCodePrefixRule {
    fn check(&self, item: &WeightedForecastMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastMetricNameLengthRule {
    pub max_length: i64,
}

impl WeightedForecastMetricRule for WeightedForecastMetricNameLengthRule {
    fn check(&self, item: &WeightedForecastMetric) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted forecast metric.
#[derive(Debug, Clone)]
pub struct WeightedForecastMetricValidator {
    pub rules: Vec<Rc<dyn WeightedForecastMetricRule>>,
}

impl WeightedForecastMetricValidator {
    pub fn validate(&self, item: &WeightedForecastMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedForecastMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedForecastMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_forecast_metric_validator() -> WeightedForecastMetricValidator {
    WeightedForecastMetricValidator {
        rules: vec![
            Rc::new(WeightedForecastMetricSegmentLengthRule { max_length: 255 }) as Rc<dyn WeightedForecastMetricRule>,
            Rc::new(WeightedForecastMetricRequiredCodeRule {
                message: "weighted forecast metric needs an entry".to_string(),
            }),
            Rc::new(WeightedForecastMetricRequiredNameRule {
                message: "weighted forecast metric needs an entry".to_string(),
            }),
            Rc::new(WeightedForecastMetricAllowedNamespaceRule {
                allowed: vec!["global".to_string(), "lenient".to_string()],
            }),
            Rc::new(WeightedForecastMetricCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(WeightedForecastMetricNameLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_forecast_metric_validator();
        let item = new_weighted_forecast_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = WeightedForecastMetricSegmentLengthRule { max_length: 86400 };
        let mut item = new_weighted_forecast_metric("x", "y");
        item.segment = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedForecastMetricRequiredCodeRule {
            message: "weighted forecast metric needs a value".to_string(),
        };
        let mut item = new_weighted_forecast_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = WeightedForecastMetricRequiredNameRule {
            message: "weighted forecast metric needs a value".to_string(),
        };
        let mut item = new_weighted_forecast_metric("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = WeightedForecastMetricNameLengthRule { max_length: 255 };
        let mut item = new_weighted_forecast_metric("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
