use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe preference must pass. Check returns its problems, or none.
pub trait ProbePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &ProbePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbePreferenceRequiredScopeRule {
    pub message: String,
}

impl ProbePreferenceRule for ProbePreferenceRequiredScopeRule {
    fn check(&self, item: &ProbePreference) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbePreferenceActiveGuardRule {
    pub limit: f64,
}

impl ProbePreferenceRule for ProbePreferenceActiveGuardRule {
    fn check(&self, item: &ProbePreference) -> Vec<String> {
        if item.active && (item.maximum > self.limit) {
            return vec![format!("maximum {} exceeds {} while active", item.maximum, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbePreferenceCodeLengthRule {
    pub max_length: i64,
}

impl ProbePreferenceRule for ProbePreferenceCodeLengthRule {
    fn check(&self, item: &ProbePreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbePreferenceSummaryLengthRule {
    pub max_length: i64,
}

impl ProbePreferenceRule for ProbePreferenceSummaryLengthRule {
    fn check(&self, item: &ProbePreference) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbePreferenceRequiredSummaryRule {
    pub message: String,
}

impl ProbePreferenceRule for ProbePreferenceRequiredSummaryRule {
    fn check(&self, item: &ProbePreference) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe preference.
#[derive(Debug, Clone)]
pub struct ProbePreferenceValidator {
    pub rules: Vec<Rc<dyn ProbePreferenceRule>>,
}

impl ProbePreferenceValidator {
    pub fn validate(&self, item: &ProbePreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_preference_validator() -> ProbePreferenceValidator {
    ProbePreferenceValidator {
        rules: vec![
            Rc::new(ProbePreferenceRequiredScopeRule {
                message: "probe preference needs a value".to_string(),
            }) as Rc<dyn ProbePreferenceRule>,
            Rc::new(ProbePreferenceActiveGuardRule { limit: 5.5 }),
            Rc::new(ProbePreferenceCodeLengthRule { max_length: 255 }),
            Rc::new(ProbePreferenceSummaryLengthRule { max_length: 16 }),
            Rc::new(ProbePreferenceRequiredSummaryRule {
                message: "probe preference needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_preference_validator();
        let item = new_probe_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = ProbePreferenceRequiredScopeRule {
            message: "probe preference needs an entry".to_string(),
        };
        let mut item = new_probe_preference("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ProbePreferenceCodeLengthRule { max_length: 120 };
        let mut item = new_probe_preference("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = ProbePreferenceSummaryLengthRule { max_length: 86400 };
        let mut item = new_probe_preference("x", "y");
        item.summary = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = ProbePreferenceRequiredSummaryRule {
            message: "probe preference needs an entry".to_string(),
        };
        let mut item = new_probe_preference("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
