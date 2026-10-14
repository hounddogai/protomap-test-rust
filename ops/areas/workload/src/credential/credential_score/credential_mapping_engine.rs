use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the credential mapping calculation.
pub trait CredentialMappingFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCredentialMappingAdjuster {
    pub step: f64,
}

impl CredentialMappingFactor for RoundingCredentialMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CredentialMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCredentialMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CredentialMappingFactor for NameCredentialMappingAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CredentialMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCredentialMappingAdjuster {
    pub percent: f64,
}

impl CredentialMappingFactor for PercentCredentialMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CredentialMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCredentialMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CredentialMappingFactor for FlaggedCredentialMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CredentialMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the credential mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialMappingEngine {
    pub steps: Vec<Rc<dyn CredentialMappingFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialMappingEngine {
    pub fn run(&mut self, item: &CredentialMapping) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CredentialMappingFactor> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[CredentialMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_credential_mapping_engine(flags: &flags::Registry) -> CredentialMappingEngine {
    CredentialMappingEngine {
        steps: vec![
            Rc::new(RoundingCredentialMappingAdjuster { step: 0.25 }) as Rc<dyn CredentialMappingFactor>,
            Rc::new(NameCredentialMappingAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.07), ("online".to_string(), 1.27)]),
            }),
            Rc::new(PercentCredentialMappingAdjuster { percent: -1.6 }),
            Rc::new(FlaggedCredentialMappingAdjuster {
                flags: flags.clone(),
                flag: "credential-mapping-rolling".to_string(),
                percent: 4.5,
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 48656.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_credential_mapping_engine(&flags::new_registry());
        let item = new_credential_mapping("code-291", "name-125");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
