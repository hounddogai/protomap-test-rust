use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary allowlist mapping calculation.
pub trait PrimaryAllowlistMappingModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryAllowlistMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryAllowlistMappingAdjuster {
    pub minimum: f64,
}

impl PrimaryAllowlistMappingModifier for MinimumPrimaryAllowlistMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryAllowlistMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePrimaryAllowlistMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryAllowlistMappingModifier for TitlePrimaryAllowlistMappingAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PrimaryAllowlistMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryAllowlistMappingAdjuster {
    pub percent: f64,
}

impl PrimaryAllowlistMappingModifier for PercentPrimaryAllowlistMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryAllowlistMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryAllowlistMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryAllowlistMappingModifier for FlaggedPrimaryAllowlistMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryAllowlistMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary allowlist mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryAllowlistMappingEngine {
    pub steps: Vec<Rc<dyn PrimaryAllowlistMappingModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryAllowlistMappingEngine {
    pub fn compute(&mut self, item: &PrimaryAllowlistMapping) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryAllowlistMappingModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryAllowlistMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_allowlist_mapping_engine(flags: &flags::Registry) -> PrimaryAllowlistMappingEngine {
    PrimaryAllowlistMappingEngine {
        steps: vec![
            Rc::new(MinimumPrimaryAllowlistMappingAdjuster { minimum: 1.6 }) as Rc<dyn PrimaryAllowlistMappingModifier>,
            Rc::new(TitlePrimaryAllowlistMappingAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.11)]),
            }),
            Rc::new(PercentPrimaryAllowlistMappingAdjuster { percent: -24.3 }),
            Rc::new(FlaggedPrimaryAllowlistMappingAdjuster {
                flags: flags.clone(),
                flag: "primary-allowlist-mapping-global".to_string(),
                percent: 2.3,
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 79360.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_allowlist_mapping_engine(&flags::new_registry());
        let item = new_primary_allowlist_mapping("code-625", "title-636");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
