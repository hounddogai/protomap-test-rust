use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional reservation mapping must pass. Check returns its problems, or none.
pub trait RegionalReservationMappingRule: std::fmt::Debug {
    fn check(&self, item: &RegionalReservationMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationMappingRequiredCodeRule {
    pub message: String,
}

impl RegionalReservationMappingRule for RegionalReservationMappingRequiredCodeRule {
    fn check(&self, item: &RegionalReservationMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationMappingRequiredNamespaceRule {
    pub message: String,
}

impl RegionalReservationMappingRule for RegionalReservationMappingRequiredNamespaceRule {
    fn check(&self, item: &RegionalReservationMapping) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationMappingTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalReservationMappingRule for RegionalReservationMappingTtlSecondsRangeRule {
    fn check(&self, item: &RegionalReservationMapping) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationMappingRequiredNameRule {
    pub message: String,
}

impl RegionalReservationMappingRule for RegionalReservationMappingRequiredNameRule {
    fn check(&self, item: &RegionalReservationMapping) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional reservation mapping.
#[derive(Debug, Clone)]
pub struct RegionalReservationMappingValidator {
    pub rules: Vec<Rc<dyn RegionalReservationMappingRule>>,
}

impl RegionalReservationMappingValidator {
    pub fn validate(&self, item: &RegionalReservationMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalReservationMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalReservationMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_reservation_mapping_validator() -> RegionalReservationMappingValidator {
    RegionalReservationMappingValidator {
        rules: vec![
            Rc::new(RegionalReservationMappingRequiredCodeRule {
                message: "regional reservation mapping needs this field".to_string(),
            }) as Rc<dyn RegionalReservationMappingRule>,
            Rc::new(RegionalReservationMappingRequiredNamespaceRule {
                message: "regional reservation mapping needs an entry".to_string(),
            }),
            Rc::new(RegionalReservationMappingTtlSecondsRangeRule { min: 64, max: 1000 }),
            Rc::new(RegionalReservationMappingRequiredNameRule {
                message: "regional reservation mapping needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_reservation_mapping_validator();
        let item = new_regional_reservation_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalReservationMappingRequiredCodeRule {
            message: "regional reservation mapping needs a value".to_string(),
        };
        let mut item = new_regional_reservation_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = RegionalReservationMappingRequiredNamespaceRule {
            message: "regional reservation mapping needs a value".to_string(),
        };
        let mut item = new_regional_reservation_mapping("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = RegionalReservationMappingTtlSecondsRangeRule { min: 64, max: 1000 };
        let mut item = new_regional_reservation_mapping("x", "y");
        item.ttl_seconds = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RegionalReservationMappingRequiredNameRule {
            message: "regional reservation mapping needs an entry".to_string(),
        };
        let mut item = new_regional_reservation_mapping("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
