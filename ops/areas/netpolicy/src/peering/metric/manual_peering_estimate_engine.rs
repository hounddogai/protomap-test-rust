use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual peering estimate calculation.
pub trait ManualPeeringEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualPeeringEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleManualPeeringEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualPeeringEstimateAdjuster for TitleManualPeeringEstimateAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ManualPeeringEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualPeeringEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualPeeringEstimateAdjuster for TieredManualPeeringEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualPeeringEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualPeeringEstimateAdjuster {
    pub cap: f64,
}

impl ManualPeeringEstimateAdjuster for CapManualPeeringEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualPeeringEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringEstimatePortOffsetAdjuster {
    pub weight: f64,
}

impl ManualPeeringEstimateAdjuster for ManualPeeringEstimatePortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &ManualPeeringEstimate, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

/// Runs the manual peering estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualPeeringEstimateEngine {
    pub steps: Vec<Rc<dyn ManualPeeringEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualPeeringEstimateEngine {
    pub fn run(&mut self, item: &ManualPeeringEstimate) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[ManualPeeringEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_peering_estimate_engine() -> ManualPeeringEstimateEngine {
    ManualPeeringEstimateEngine {
        steps: vec![
            Rc::new(TitleManualPeeringEstimateAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.39), ("south".to_string(), 1.38)]),
            }) as Rc<dyn ManualPeeringEstimateAdjuster>,
            Rc::new(TieredManualPeeringEstimateAdjuster {
                bounds: vec![4330.0, 6383.0],
                rates: vec![1.17, 1.06],
            }),
            Rc::new(CapManualPeeringEstimateAdjuster { cap: 45322.0 }),
            Rc::new(ManualPeeringEstimatePortOffsetAdjuster { weight: 0.02 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 37188.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_peering_estimate_engine();
        let item = new_manual_peering_estimate("code-68", "title-194");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
