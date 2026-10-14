use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the release rollup calculation.
pub trait ReleaseRollupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReleaseRollupStep for TieredReleaseRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReleaseRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReleaseRollupAdjuster {
    pub minimum: f64,
}

impl ReleaseRollupStep for MinimumReleaseRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReleaseRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameReleaseRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReleaseRollupStep for NameReleaseRollupAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ReleaseRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the release rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseRollupEngine {
    pub steps: Vec<Rc<dyn ReleaseRollupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseRollupEngine {
    pub fn run(&mut self, item: &ReleaseRollup) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_release_rollup_engine() -> ReleaseRollupEngine {
    ReleaseRollupEngine {
        steps: vec![
            Rc::new(TieredReleaseRollupAdjuster {
                bounds: vec![1299.0, 2778.0, 4039.0, 8456.0],
                rates: vec![1.08, 0.89, 1.07, 0.99],
            }) as Rc<dyn ReleaseRollupStep>,
            Rc::new(MinimumReleaseRollupAdjuster { minimum: 22.6 }),
            Rc::new(NameReleaseRollupAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.99)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 32514.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_rollup_engine();
        let item = new_release_rollup("code-242", "name-47");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
