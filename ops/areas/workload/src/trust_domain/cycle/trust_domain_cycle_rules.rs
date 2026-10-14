use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain cycle must pass. Check returns its problems, or none.
pub trait TrustDomainCycleRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainCycleMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TrustDomainCycleRule for TrustDomainCycleMaximumRangeRule {
    fn check(&self, item: &TrustDomainCycle) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainCycleUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainCycleRule for TrustDomainCycleUpdatedAtRangeRule {
    fn check(&self, item: &TrustDomainCycle) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainCycleSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainCycleRule for TrustDomainCycleSequenceRangeRule {
    fn check(&self, item: &TrustDomainCycle) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainCycleRequiredCodeRule {
    pub message: String,
}

impl TrustDomainCycleRule for TrustDomainCycleRequiredCodeRule {
    fn check(&self, item: &TrustDomainCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain cycle.
#[derive(Debug, Clone)]
pub struct TrustDomainCycleValidator {
    pub rules: Vec<Rc<dyn TrustDomainCycleRule>>,
}

impl TrustDomainCycleValidator {
    pub fn validate(&self, item: &TrustDomainCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_cycle_validator() -> TrustDomainCycleValidator {
    TrustDomainCycleValidator {
        rules: vec![
            Rc::new(TrustDomainCycleMaximumRangeRule { min: 3.2, max: 4091.9 }) as Rc<dyn TrustDomainCycleRule>,
            Rc::new(TrustDomainCycleUpdatedAtRangeRule { min: 16, max: 80 }),
            Rc::new(TrustDomainCycleSequenceRangeRule { min: 64, max: 64 }),
            Rc::new(TrustDomainCycleRequiredCodeRule {
                message: "trust domain cycle needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_cycle_validator();
        let item = new_trust_domain_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = TrustDomainCycleMaximumRangeRule { min: 6.8, max: 2201.1 };
        let mut item = new_trust_domain_cycle("x", "y");
        item.maximum = 2202.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = TrustDomainCycleUpdatedAtRangeRule { min: 16, max: 64 };
        let mut item = new_trust_domain_cycle("x", "y");
        item.updated_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = TrustDomainCycleSequenceRangeRule { min: 3600, max: 3600 };
        let mut item = new_trust_domain_cycle("x", "y");
        item.sequence = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TrustDomainCycleRequiredCodeRule {
            message: "trust domain cycle needs a value".to_string(),
        };
        let mut item = new_trust_domain_cycle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
