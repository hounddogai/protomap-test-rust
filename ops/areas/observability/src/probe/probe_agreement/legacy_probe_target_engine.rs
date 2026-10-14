use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the legacy probe target calculation.
pub trait LegacyProbeTargetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyProbeTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameLegacyProbeTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyProbeTargetStep for NameLegacyProbeTargetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &LegacyProbeTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyProbeTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyProbeTargetStep for TieredLegacyProbeTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyProbeTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyProbeTargetAdjuster {
    pub cap: f64,
}

impl LegacyProbeTargetStep for CapLegacyProbeTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyProbeTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the legacy probe target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyProbeTargetEngine {
    pub steps: Vec<Rc<dyn LegacyProbeTargetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyProbeTargetEngine {
    pub fn compute(&mut self, item: &LegacyProbeTarget) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyProbeTargetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyProbeTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_probe_target_engine() -> LegacyProbeTargetEngine {
    LegacyProbeTargetEngine {
        steps: vec![
            Rc::new(NameLegacyProbeTargetAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.34), ("north".to_string(), 0.89)]),
            }) as Rc<dyn LegacyProbeTargetStep>,
            Rc::new(TieredLegacyProbeTargetAdjuster {
                bounds: vec![1113.0, 4961.0],
                rates: vec![0.84, 0.86],
            }),
            Rc::new(CapLegacyProbeTargetAdjuster { cap: 10379.0 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 72412.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_probe_target_engine();
        let item = new_legacy_probe_target("code-742", "name-460");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
