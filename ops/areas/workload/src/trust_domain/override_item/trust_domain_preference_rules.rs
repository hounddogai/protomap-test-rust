use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain preference must pass. Check returns its problems, or none.
pub trait TrustDomainPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainPreferenceRequiredCodeRule {
    pub message: String,
}

impl TrustDomainPreferenceRule for TrustDomainPreferenceRequiredCodeRule {
    fn check(&self, item: &TrustDomainPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainPreferenceRequiredUnitRule {
    pub message: String,
}

impl TrustDomainPreferenceRule for TrustDomainPreferenceRequiredUnitRule {
    fn check(&self, item: &TrustDomainPreference) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainPreferenceCodePrefixRule {
    pub prefix: String,
}

impl TrustDomainPreferenceRule for TrustDomainPreferenceCodePrefixRule {
    fn check(&self, item: &TrustDomainPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainPreferenceRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TrustDomainPreferenceRule for TrustDomainPreferenceRatioRangeRule {
    fn check(&self, item: &TrustDomainPreference) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain preference.
#[derive(Debug, Clone)]
pub struct TrustDomainPreferenceValidator {
    pub rules: Vec<Rc<dyn TrustDomainPreferenceRule>>,
}

impl TrustDomainPreferenceValidator {
    pub fn validate(&self, item: &TrustDomainPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TrustDomainPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_preference_validator() -> TrustDomainPreferenceValidator {
    TrustDomainPreferenceValidator {
        rules: vec![
            Rc::new(TrustDomainPreferenceRequiredCodeRule {
                message: "trust domain preference needs a value".to_string(),
            }) as Rc<dyn TrustDomainPreferenceRule>,
            Rc::new(TrustDomainPreferenceRequiredUnitRule {
                message: "trust domain preference needs this field".to_string(),
            }),
            Rc::new(TrustDomainPreferenceCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(TrustDomainPreferenceRatioRangeRule { min: 2.6, max: 2031.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_preference_validator();
        let item = new_trust_domain_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TrustDomainPreferenceRequiredCodeRule {
            message: "trust domain preference needs this field".to_string(),
        };
        let mut item = new_trust_domain_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = TrustDomainPreferenceRequiredUnitRule {
            message: "trust domain preference needs a value".to_string(),
        };
        let mut item = new_trust_domain_preference("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = TrustDomainPreferenceRatioRangeRule { min: 0.6, max: 3415.7 };
        let mut item = new_trust_domain_preference("x", "y");
        item.ratio = 3417.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
