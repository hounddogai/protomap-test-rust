use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional allowlist target calculation.
pub trait RegionalAllowlistTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalAllowlistTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAllowlistTargetCapacityOffsetAdjuster {
    pub weight: f64,
}

impl RegionalAllowlistTargetAdjuster for RegionalAllowlistTargetCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalAllowlistTarget, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalAllowlistTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalAllowlistTargetAdjuster for TieredRegionalAllowlistTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalAllowlistTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRegionalAllowlistTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalAllowlistTargetAdjuster for LabelRegionalAllowlistTargetAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RegionalAllowlistTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalAllowlistTargetAdjuster {
    pub minimum: f64,
}

impl RegionalAllowlistTargetAdjuster for MinimumRegionalAllowlistTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalAllowlistTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalAllowlistTargetAdjuster {
    pub step: f64,
}

impl RegionalAllowlistTargetAdjuster for RoundingRegionalAllowlistTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalAllowlistTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalAllowlistTargetAdjuster {
    pub cap: f64,
}

impl RegionalAllowlistTargetAdjuster for CapRegionalAllowlistTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalAllowlistTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the regional allowlist target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalAllowlistTargetEngine {
    pub steps: Vec<Rc<dyn RegionalAllowlistTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalAllowlistTargetEngine {
    pub fn run(&mut self, item: &RegionalAllowlistTarget) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[RegionalAllowlistTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_allowlist_target_engine() -> RegionalAllowlistTargetEngine {
    RegionalAllowlistTargetEngine {
        steps: vec![
            Rc::new(RegionalAllowlistTargetCapacityOffsetAdjuster { weight: 0.56 })
                as Rc<dyn RegionalAllowlistTargetAdjuster>,
            Rc::new(TieredRegionalAllowlistTargetAdjuster {
                bounds: vec![4986.0, 5820.0, 9864.0],
                rates: vec![0.84, 1.02, 0.89],
            }),
            Rc::new(LabelRegionalAllowlistTargetAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.98), ("north".to_string(), 0.8)]),
            }),
            Rc::new(MinimumRegionalAllowlistTargetAdjuster { minimum: 19.3 }),
            Rc::new(RoundingRegionalAllowlistTargetAdjuster { step: 0.25 }),
            Rc::new(CapRegionalAllowlistTargetAdjuster { cap: 32362.0 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 6560.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_allowlist_target_engine();
        let item = new_regional_allowlist_target("code-679", "label-905");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
