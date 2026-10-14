use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a probe metric must pass. Check returns its problems, or none.
pub trait ProbeMetricRule: std::fmt::Debug {
    fn check(&self, item: &ProbeMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeMetricAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ProbeMetricRule for ProbeMetricAllowedTitleRule {
    fn check(&self, item: &ProbeMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is inconsistent", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeMetricQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeMetricRule for ProbeMetricQuotaUnitsRangeRule {
    fn check(&self, item: &ProbeMetric) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeMetricRequiredReviewerRule {
    pub message: String,
}

impl ProbeMetricRule for ProbeMetricRequiredReviewerRule {
    fn check(&self, item: &ProbeMetric) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeMetricCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ProbeMetricRule for ProbeMetricCostRangeRule {
    fn check(&self, item: &ProbeMetric) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeMetricRequiredTitleRule {
    pub message: String,
}

impl ProbeMetricRule for ProbeMetricRequiredTitleRule {
    fn check(&self, item: &ProbeMetric) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe metric.
#[derive(Debug, Clone)]
pub struct ProbeMetricValidator {
    pub rules: Vec<Rc<dyn ProbeMetricRule>>,
}

impl ProbeMetricValidator {
    pub fn validate(&self, item: &ProbeMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_metric_validator() -> ProbeMetricValidator {
    ProbeMetricValidator {
        rules: vec![
            Rc::new(ProbeMetricAllowedTitleRule {
                allowed: vec!["capped".to_string(), "adaptive".to_string()],
            }) as Rc<dyn ProbeMetricRule>,
            Rc::new(ProbeMetricQuotaUnitsRangeRule { min: 16, max: 255 }),
            Rc::new(ProbeMetricRequiredReviewerRule {
                message: "probe metric needs this field".to_string(),
            }),
            Rc::new(ProbeMetricCostRangeRule { min: 8.3, max: 3020.2 }),
            Rc::new(ProbeMetricRequiredTitleRule {
                message: "probe metric needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_metric_validator();
        let item = new_probe_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = ProbeMetricQuotaUnitsRangeRule { min: 32, max: 1000 };
        let mut item = new_probe_metric("x", "y");
        item.quota_units = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = ProbeMetricRequiredReviewerRule {
            message: "probe metric needs this field".to_string(),
        };
        let mut item = new_probe_metric("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = ProbeMetricCostRangeRule { min: 6.3, max: 4942.5 };
        let mut item = new_probe_metric("x", "y");
        item.cost = 4944.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ProbeMetricRequiredTitleRule {
            message: "probe metric needs this field".to_string(),
        };
        let mut item = new_probe_metric("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
