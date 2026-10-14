use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the health check entry calculation.
pub trait HealthCheckEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapHealthCheckEntryAdjuster {
    pub cap: f64,
}

impl HealthCheckEntryAdjuster for CapHealthCheckEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &HealthCheckEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl HealthCheckEntryAdjuster for TieredHealthCheckEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &HealthCheckEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentHealthCheckEntryAdjuster {
    pub percent: f64,
}

impl HealthCheckEntryAdjuster for PercentHealthCheckEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &HealthCheckEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the health check entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckEntryEngine {
    pub steps: Vec<Rc<dyn HealthCheckEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckEntryEngine {
    pub fn evaluate(&mut self, item: &HealthCheckEntry) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn HealthCheckEntryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_health_check_entry_engine() -> HealthCheckEntryEngine {
    HealthCheckEntryEngine {
        steps: vec![
            Rc::new(CapHealthCheckEntryAdjuster { cap: 48585.0 }) as Rc<dyn HealthCheckEntryAdjuster>,
            Rc::new(TieredHealthCheckEntryAdjuster {
                bounds: vec![4157.0, 8335.0],
                rates: vec![1.02, 1.18],
            }),
            Rc::new(PercentHealthCheckEntryAdjuster { percent: 8.6 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 38200.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_health_check_entry_engine();
        let item = new_health_check_entry("code-767", "label-734");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
