use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the pending canary mapping calculation.
pub trait PendingCanaryMappingFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingCanaryMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryPendingCanaryMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingCanaryMappingFactor for CountryPendingCanaryMappingAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &PendingCanaryMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingCanaryMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingCanaryMappingFactor for TieredPendingCanaryMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingCanaryMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingCanaryMappingAdjuster {
    pub percent: f64,
}

impl PendingCanaryMappingFactor for PercentPendingCanaryMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingCanaryMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending canary mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingCanaryMappingEngine {
    pub steps: Vec<Rc<dyn PendingCanaryMappingFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingCanaryMappingEngine {
    pub fn price(&mut self, item: &PendingCanaryMapping) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingCanaryMappingFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingCanaryMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_canary_mapping_engine() -> PendingCanaryMappingEngine {
    PendingCanaryMappingEngine {
        steps: vec![
            Rc::new(CountryPendingCanaryMappingAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.34),
                    ("north".to_string(), 1.01),
                    ("central".to_string(), 0.86),
                ]),
            }) as Rc<dyn PendingCanaryMappingFactor>,
            Rc::new(TieredPendingCanaryMappingAdjuster {
                bounds: vec![599.0, 3274.0],
                rates: vec![1.04, 0.98],
            }),
            Rc::new(PercentPendingCanaryMappingAdjuster { percent: -16.1 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 73458.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_canary_mapping_engine();
        let item = new_pending_canary_mapping("code-489", "title-190");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
