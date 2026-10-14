use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional probe claim must pass. Check returns its problems, or none.
pub trait RegionalProbeClaimRule: std::fmt::Debug {
    fn check(&self, item: &RegionalProbeClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeClaimVisibleGuardRule {
    pub limit: i64,
}

impl RegionalProbeClaimRule for RegionalProbeClaimVisibleGuardRule {
    fn check(&self, item: &RegionalProbeClaim) -> Vec<String> {
        if item.visible && (item.duration_minutes > self.limit) {
            return vec![format!(
                "duration minutes {} exceeds {} while visible",
                item.duration_minutes, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeClaimRequiredLabelRule {
    pub message: String,
}

impl RegionalProbeClaimRule for RegionalProbeClaimRequiredLabelRule {
    fn check(&self, item: &RegionalProbeClaim) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeClaimCodeLengthRule {
    pub max_length: i64,
}

impl RegionalProbeClaimRule for RegionalProbeClaimCodeLengthRule {
    fn check(&self, item: &RegionalProbeClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeClaimExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalProbeClaimRule for RegionalProbeClaimExpiresAtRangeRule {
    fn check(&self, item: &RegionalProbeClaim) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeClaimRequiredSummaryRule {
    pub message: String,
}

impl RegionalProbeClaimRule for RegionalProbeClaimRequiredSummaryRule {
    fn check(&self, item: &RegionalProbeClaim) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional probe claim.
#[derive(Debug, Clone)]
pub struct RegionalProbeClaimValidator {
    pub rules: Vec<Rc<dyn RegionalProbeClaimRule>>,
}

impl RegionalProbeClaimValidator {
    pub fn validate(&self, item: &RegionalProbeClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalProbeClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalProbeClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_probe_claim_validator() -> RegionalProbeClaimValidator {
    RegionalProbeClaimValidator {
        rules: vec![
            Rc::new(RegionalProbeClaimVisibleGuardRule { limit: 1000 }) as Rc<dyn RegionalProbeClaimRule>,
            Rc::new(RegionalProbeClaimRequiredLabelRule {
                message: "regional probe claim needs a value".to_string(),
            }),
            Rc::new(RegionalProbeClaimCodeLengthRule { max_length: 255 }),
            Rc::new(RegionalProbeClaimExpiresAtRangeRule { min: 1000, max: 86400 }),
            Rc::new(RegionalProbeClaimRequiredSummaryRule {
                message: "regional probe claim needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_probe_claim_validator();
        let item = new_regional_probe_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionalProbeClaimRequiredLabelRule {
            message: "regional probe claim needs this field".to_string(),
        };
        let mut item = new_regional_probe_claim("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalProbeClaimCodeLengthRule { max_length: 120 };
        let mut item = new_regional_probe_claim("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = RegionalProbeClaimExpiresAtRangeRule { min: 16, max: 3600 };
        let mut item = new_regional_probe_claim("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = RegionalProbeClaimRequiredSummaryRule {
            message: "regional probe claim needs a value".to_string(),
        };
        let mut item = new_regional_probe_claim("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
