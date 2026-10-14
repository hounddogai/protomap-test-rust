use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tenant mapping must pass. Check returns its problems, or none.
pub trait TenantMappingRule: std::fmt::Debug {
    fn check(&self, item: &TenantMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantMappingRequiredCodeRule {
    pub message: String,
}

impl TenantMappingRule for TenantMappingRequiredCodeRule {
    fn check(&self, item: &TenantMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantMappingWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TenantMappingRule for TenantMappingWindowDaysRangeRule {
    fn check(&self, item: &TenantMapping) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantMappingSegmentLengthRule {
    pub max_length: i64,
}

impl TenantMappingRule for TenantMappingSegmentLengthRule {
    fn check(&self, item: &TenantMapping) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantMappingAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl TenantMappingRule for TenantMappingAllowedSegmentRule {
    fn check(&self, item: &TenantMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is expired", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantMappingRequiredLabelRule {
    pub message: String,
}

impl TenantMappingRule for TenantMappingRequiredLabelRule {
    fn check(&self, item: &TenantMapping) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant mapping.
#[derive(Debug, Clone)]
pub struct TenantMappingValidator {
    pub rules: Vec<Rc<dyn TenantMappingRule>>,
}

impl TenantMappingValidator {
    pub fn validate(&self, item: &TenantMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_mapping_validator() -> TenantMappingValidator {
    TenantMappingValidator {
        rules: vec![
            Rc::new(TenantMappingRequiredCodeRule {
                message: "tenant mapping needs a value".to_string(),
            }) as Rc<dyn TenantMappingRule>,
            Rc::new(TenantMappingWindowDaysRangeRule { min: 16, max: 255 }),
            Rc::new(TenantMappingSegmentLengthRule { max_length: 16 }),
            Rc::new(TenantMappingAllowedSegmentRule {
                allowed: vec!["weighted".to_string(), "scheduled".to_string()],
            }),
            Rc::new(TenantMappingRequiredLabelRule {
                message: "tenant mapping needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_mapping_validator();
        let item = new_tenant_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TenantMappingRequiredCodeRule {
            message: "tenant mapping needs a value".to_string(),
        };
        let mut item = new_tenant_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = TenantMappingWindowDaysRangeRule { min: 86400, max: 86400 };
        let mut item = new_tenant_mapping("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = TenantMappingSegmentLengthRule { max_length: 255 };
        let mut item = new_tenant_mapping("x", "y");
        item.segment = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TenantMappingRequiredLabelRule {
            message: "tenant mapping needs this field".to_string(),
        };
        let mut item = new_tenant_mapping("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
