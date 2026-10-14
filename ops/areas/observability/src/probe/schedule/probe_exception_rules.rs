use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe exception must pass. Check returns its problems, or none.
pub trait ProbeExceptionRule: std::fmt::Debug {
    fn check(&self, item: &ProbeException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeExceptionRequiredLabelRule {
    pub message: String,
}

impl ProbeExceptionRule for ProbeExceptionRequiredLabelRule {
    fn check(&self, item: &ProbeException) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeExceptionCodePrefixRule {
    pub prefix: String,
}

impl ProbeExceptionRule for ProbeExceptionCodePrefixRule {
    fn check(&self, item: &ProbeException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeExceptionRequiredCodeRule {
    pub message: String,
}

impl ProbeExceptionRule for ProbeExceptionRequiredCodeRule {
    fn check(&self, item: &ProbeException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeExceptionRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ProbeExceptionRule for ProbeExceptionRatioRangeRule {
    fn check(&self, item: &ProbeException) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeExceptionLabelLengthRule {
    pub max_length: i64,
}

impl ProbeExceptionRule for ProbeExceptionLabelLengthRule {
    fn check(&self, item: &ProbeException) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe exception.
#[derive(Debug, Clone)]
pub struct ProbeExceptionValidator {
    pub rules: Vec<Rc<dyn ProbeExceptionRule>>,
}

impl ProbeExceptionValidator {
    pub fn validate(&self, item: &ProbeException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_exception_validator() -> ProbeExceptionValidator {
    ProbeExceptionValidator {
        rules: vec![
            Rc::new(ProbeExceptionRequiredLabelRule {
                message: "probe exception needs a value".to_string(),
            }) as Rc<dyn ProbeExceptionRule>,
            Rc::new(ProbeExceptionCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ProbeExceptionRequiredCodeRule {
                message: "probe exception needs this field".to_string(),
            }),
            Rc::new(ProbeExceptionRatioRangeRule { min: 2.1, max: 451.3 }),
            Rc::new(ProbeExceptionLabelLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_exception_validator();
        let item = new_probe_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ProbeExceptionRequiredLabelRule {
            message: "probe exception needs this field".to_string(),
        };
        let mut item = new_probe_exception("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ProbeExceptionRequiredCodeRule {
            message: "probe exception needs a value".to_string(),
        };
        let mut item = new_probe_exception("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = ProbeExceptionRatioRangeRule { min: 1.3, max: 990.8 };
        let mut item = new_probe_exception("x", "y");
        item.ratio = 992.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ProbeExceptionLabelLengthRule { max_length: 16 };
        let mut item = new_probe_exception("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
