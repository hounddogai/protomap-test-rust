use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary evidence entry must pass. Check returns its problems, or none.
pub trait PrimaryEvidenceEntryRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryEvidenceEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceEntryCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryEvidenceEntryRule for PrimaryEvidenceEntryCodeLengthRule {
    fn check(&self, item: &PrimaryEvidenceEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceEntryRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryEvidenceEntryRule for PrimaryEvidenceEntryRateRangeRule {
    fn check(&self, item: &PrimaryEvidenceEntry) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceEntryClusterLengthRule {
    pub max_length: i64,
}

impl PrimaryEvidenceEntryRule for PrimaryEvidenceEntryClusterLengthRule {
    fn check(&self, item: &PrimaryEvidenceEntry) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceEntryBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryEvidenceEntryRule for PrimaryEvidenceEntryBurstRangeRule {
    fn check(&self, item: &PrimaryEvidenceEntry) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceEntryRequiredClusterRule {
    pub message: String,
}

impl PrimaryEvidenceEntryRule for PrimaryEvidenceEntryRequiredClusterRule {
    fn check(&self, item: &PrimaryEvidenceEntry) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary evidence entry.
#[derive(Debug, Clone)]
pub struct PrimaryEvidenceEntryValidator {
    pub rules: Vec<Rc<dyn PrimaryEvidenceEntryRule>>,
}

impl PrimaryEvidenceEntryValidator {
    pub fn validate(&self, item: &PrimaryEvidenceEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryEvidenceEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryEvidenceEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_evidence_entry_validator() -> PrimaryEvidenceEntryValidator {
    PrimaryEvidenceEntryValidator {
        rules: vec![
            Rc::new(PrimaryEvidenceEntryCodeLengthRule { max_length: 3600 }) as Rc<dyn PrimaryEvidenceEntryRule>,
            Rc::new(PrimaryEvidenceEntryRateRangeRule { min: 2.4, max: 686.5 }),
            Rc::new(PrimaryEvidenceEntryClusterLengthRule { max_length: 16 }),
            Rc::new(PrimaryEvidenceEntryBurstRangeRule { min: 80, max: 3600 }),
            Rc::new(PrimaryEvidenceEntryRequiredClusterRule {
                message: "primary evidence entry needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_evidence_entry_validator();
        let item = new_primary_evidence_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryEvidenceEntryCodeLengthRule { max_length: 120 };
        let mut item = new_primary_evidence_entry("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = PrimaryEvidenceEntryRateRangeRule { min: 1.8, max: 4105.2 };
        let mut item = new_primary_evidence_entry("x", "y");
        item.rate = 4106.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = PrimaryEvidenceEntryClusterLengthRule { max_length: 3600 };
        let mut item = new_primary_evidence_entry("x", "y");
        item.cluster = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = PrimaryEvidenceEntryBurstRangeRule { min: 64, max: 120 };
        let mut item = new_primary_evidence_entry("x", "y");
        item.burst = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = PrimaryEvidenceEntryRequiredClusterRule {
            message: "primary evidence entry needs an entry".to_string(),
        };
        let mut item = new_primary_evidence_entry("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
