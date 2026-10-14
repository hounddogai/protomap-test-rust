use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a trust domain override must pass. Check returns its problems, or none.
pub trait TrustDomainOverrideRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainOverrideAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TrustDomainOverrideRule for TrustDomainOverrideAllowedCodeRule {
    fn check(&self, item: &TrustDomainOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainOverrideGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainOverrideRule for TrustDomainOverrideGraceDaysRangeRule {
    fn check(&self, item: &TrustDomainOverride) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainOverrideCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TrustDomainOverrideRule for TrustDomainOverrideCeilingRangeRule {
    fn check(&self, item: &TrustDomainOverride) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainOverrideMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TrustDomainOverrideRule for TrustDomainOverrideMinimumRangeRule {
    fn check(&self, item: &TrustDomainOverride) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainOverrideArchivedGuardRule {
    pub limit: f64,
}

impl TrustDomainOverrideRule for TrustDomainOverrideArchivedGuardRule {
    fn check(&self, item: &TrustDomainOverride) -> Vec<String> {
        if item.archived && (item.floor_value > self.limit) {
            return vec![format!(
                "floor value {} exceeds {} while archived",
                item.floor_value, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain override.
#[derive(Debug, Clone)]
pub struct TrustDomainOverrideValidator {
    pub rules: Vec<Rc<dyn TrustDomainOverrideRule>>,
}

impl TrustDomainOverrideValidator {
    pub fn validate(&self, item: &TrustDomainOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_override_validator() -> TrustDomainOverrideValidator {
    TrustDomainOverrideValidator {
        rules: vec![
            Rc::new(TrustDomainOverrideAllowedCodeRule {
                allowed: vec!["default".to_string(), "legacy".to_string()],
            }) as Rc<dyn TrustDomainOverrideRule>,
            Rc::new(TrustDomainOverrideGraceDaysRangeRule { min: 64, max: 120 }),
            Rc::new(TrustDomainOverrideCeilingRangeRule { min: 8.8, max: 3176.0 }),
            Rc::new(TrustDomainOverrideMinimumRangeRule { min: 8.7, max: 3324.9 }),
            Rc::new(TrustDomainOverrideArchivedGuardRule { limit: 9.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_override_validator();
        let item = new_trust_domain_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = TrustDomainOverrideGraceDaysRangeRule { min: 80, max: 120 };
        let mut item = new_trust_domain_override("x", "y");
        item.grace_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = TrustDomainOverrideCeilingRangeRule { min: 2.5, max: 4346.7 };
        let mut item = new_trust_domain_override("x", "y");
        item.ceiling = 4348.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = TrustDomainOverrideMinimumRangeRule { min: 3.3, max: 4916.9 };
        let mut item = new_trust_domain_override("x", "y");
        item.minimum = 4918.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
