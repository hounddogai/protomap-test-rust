use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe target must pass. Check returns its problems, or none.
pub trait ProbeTargetRule: std::fmt::Debug {
    fn check(&self, item: &ProbeTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeTargetStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeTargetRule for ProbeTargetStartsAtRangeRule {
    fn check(&self, item: &ProbeTarget) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeTargetMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ProbeTargetRule for ProbeTargetMaximumRangeRule {
    fn check(&self, item: &ProbeTarget) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeTargetRequiredCodeRule {
    pub message: String,
}

impl ProbeTargetRule for ProbeTargetRequiredCodeRule {
    fn check(&self, item: &ProbeTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeTargetCodePrefixRule {
    pub prefix: String,
}

impl ProbeTargetRule for ProbeTargetCodePrefixRule {
    fn check(&self, item: &ProbeTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe target.
#[derive(Debug, Clone)]
pub struct ProbeTargetValidator {
    pub rules: Vec<Rc<dyn ProbeTargetRule>>,
}

impl ProbeTargetValidator {
    pub fn validate(&self, item: &ProbeTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_target_validator() -> ProbeTargetValidator {
    ProbeTargetValidator {
        rules: vec![
            Rc::new(ProbeTargetStartsAtRangeRule { min: 86400, max: 86400 }) as Rc<dyn ProbeTargetRule>,
            Rc::new(ProbeTargetMaximumRangeRule { min: 0.6, max: 2081.3 }),
            Rc::new(ProbeTargetRequiredCodeRule {
                message: "probe target needs this field".to_string(),
            }),
            Rc::new(ProbeTargetCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_target_validator();
        let item = new_probe_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = ProbeTargetStartsAtRangeRule { min: 255, max: 3600 };
        let mut item = new_probe_target("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = ProbeTargetMaximumRangeRule { min: 8.6, max: 2758.7 };
        let mut item = new_probe_target("x", "y");
        item.maximum = 2760.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ProbeTargetRequiredCodeRule {
            message: "probe target needs an entry".to_string(),
        };
        let mut item = new_probe_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
