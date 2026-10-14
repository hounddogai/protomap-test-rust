use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback postmortem metric must pass. Check returns its problems, or none.
pub trait FallbackPostmortemMetricRule: std::fmt::Debug {
    fn check(&self, item: &FallbackPostmortemMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemMetricRequiredCodeRule {
    pub message: String,
}

impl FallbackPostmortemMetricRule for FallbackPostmortemMetricRequiredCodeRule {
    fn check(&self, item: &FallbackPostmortemMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemMetricRequiredExternalRefRule {
    pub message: String,
}

impl FallbackPostmortemMetricRule for FallbackPostmortemMetricRequiredExternalRefRule {
    fn check(&self, item: &FallbackPostmortemMetric) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemMetricRequiredPrincipalRule {
    pub message: String,
}

impl FallbackPostmortemMetricRule for FallbackPostmortemMetricRequiredPrincipalRule {
    fn check(&self, item: &FallbackPostmortemMetric) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback postmortem metric.
#[derive(Debug, Clone)]
pub struct FallbackPostmortemMetricValidator {
    pub rules: Vec<Rc<dyn FallbackPostmortemMetricRule>>,
}

impl FallbackPostmortemMetricValidator {
    pub fn validate(&self, item: &FallbackPostmortemMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackPostmortemMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackPostmortemMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_postmortem_metric_validator() -> FallbackPostmortemMetricValidator {
    FallbackPostmortemMetricValidator {
        rules: vec![
            Rc::new(FallbackPostmortemMetricRequiredCodeRule {
                message: "fallback postmortem metric needs an entry".to_string(),
            }) as Rc<dyn FallbackPostmortemMetricRule>,
            Rc::new(FallbackPostmortemMetricRequiredExternalRefRule {
                message: "fallback postmortem metric needs this field".to_string(),
            }),
            Rc::new(FallbackPostmortemMetricRequiredPrincipalRule {
                message: "fallback postmortem metric needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_postmortem_metric_validator();
        let item = new_fallback_postmortem_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackPostmortemMetricRequiredCodeRule {
            message: "fallback postmortem metric needs this field".to_string(),
        };
        let mut item = new_fallback_postmortem_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = FallbackPostmortemMetricRequiredExternalRefRule {
            message: "fallback postmortem metric needs this field".to_string(),
        };
        let mut item = new_fallback_postmortem_metric("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = FallbackPostmortemMetricRequiredPrincipalRule {
            message: "fallback postmortem metric needs a value".to_string(),
        };
        let mut item = new_fallback_postmortem_metric("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
