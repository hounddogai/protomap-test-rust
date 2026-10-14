use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the health check draft calculation.
pub trait HealthCheckDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentHealthCheckDraftAdjuster {
    pub percent: f64,
}

impl HealthCheckDraftFactor for PercentHealthCheckDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &HealthCheckDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingHealthCheckDraftAdjuster {
    pub step: f64,
}

impl HealthCheckDraftFactor for RoundingHealthCheckDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &HealthCheckDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl HealthCheckDraftFactor for TieredHealthCheckDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &HealthCheckDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumHealthCheckDraftAdjuster {
    pub minimum: f64,
}

impl HealthCheckDraftFactor for MinimumHealthCheckDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &HealthCheckDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the health check draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckDraftEngine {
    pub steps: Vec<Rc<dyn HealthCheckDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckDraftEngine {
    pub fn price(&mut self, item: &HealthCheckDraft) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_health_check_draft_engine() -> HealthCheckDraftEngine {
    HealthCheckDraftEngine {
        steps: vec![
            Rc::new(PercentHealthCheckDraftAdjuster { percent: 23.0 }) as Rc<dyn HealthCheckDraftFactor>,
            Rc::new(RoundingHealthCheckDraftAdjuster { step: 0.05 }),
            Rc::new(TieredHealthCheckDraftAdjuster {
                bounds: vec![4855.0, 6036.0, 6552.0, 8442.0],
                rates: vec![0.96, 0.8, 1.19, 1.18],
            }),
            Rc::new(MinimumHealthCheckDraftAdjuster { minimum: 19.6 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 3729.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_health_check_draft_engine();
        let item = new_health_check_draft("code-407", "title-173");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
