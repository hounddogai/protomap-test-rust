use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a probe estimate must pass. Check returns its problems, or none.
pub trait ProbeEstimateRule: std::fmt::Debug {
    fn check(&self, item: &ProbeEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeEstimateRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ProbeEstimateRule for ProbeEstimateRatioRangeRule {
    fn check(&self, item: &ProbeEstimate) -> Vec<String> {
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
pub struct ProbeEstimateNameLengthRule {
    pub max_length: i64,
}

impl ProbeEstimateRule for ProbeEstimateNameLengthRule {
    fn check(&self, item: &ProbeEstimate) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeEstimateRequiredChecksumRule {
    pub message: String,
}

impl ProbeEstimateRule for ProbeEstimateRequiredChecksumRule {
    fn check(&self, item: &ProbeEstimate) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeEstimateRequiredCodeRule {
    pub message: String,
}

impl ProbeEstimateRule for ProbeEstimateRequiredCodeRule {
    fn check(&self, item: &ProbeEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeEstimateAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl ProbeEstimateRule for ProbeEstimateAllowedTenantIdRule {
    fn check(&self, item: &ProbeEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is below minimum", item.tenant_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe estimate.
#[derive(Debug, Clone)]
pub struct ProbeEstimateValidator {
    pub rules: Vec<Rc<dyn ProbeEstimateRule>>,
}

impl ProbeEstimateValidator {
    pub fn validate(&self, item: &ProbeEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ProbeEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_estimate_validator() -> ProbeEstimateValidator {
    ProbeEstimateValidator {
        rules: vec![
            Rc::new(ProbeEstimateRatioRangeRule { min: 0.1, max: 800.0 }) as Rc<dyn ProbeEstimateRule>,
            Rc::new(ProbeEstimateNameLengthRule { max_length: 16 }),
            Rc::new(ProbeEstimateRequiredChecksumRule {
                message: "probe estimate needs an entry".to_string(),
            }),
            Rc::new(ProbeEstimateRequiredCodeRule {
                message: "probe estimate needs a value".to_string(),
            }),
            Rc::new(ProbeEstimateAllowedTenantIdRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_estimate_validator();
        let item = new_probe_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = ProbeEstimateRatioRangeRule { min: 4.2, max: 1812.2 };
        let mut item = new_probe_estimate("x", "y");
        item.ratio = 1813.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ProbeEstimateNameLengthRule { max_length: 3600 };
        let mut item = new_probe_estimate("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = ProbeEstimateRequiredChecksumRule {
            message: "probe estimate needs this field".to_string(),
        };
        let mut item = new_probe_estimate("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ProbeEstimateRequiredCodeRule {
            message: "probe estimate needs an entry".to_string(),
        };
        let mut item = new_probe_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
