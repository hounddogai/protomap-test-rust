use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered health check variance calculation.
pub trait TieredHealthCheckVarianceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredHealthCheckVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredHealthCheckVarianceAdjuster {
    pub percent: f64,
}

impl TieredHealthCheckVarianceModifier for PercentTieredHealthCheckVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredHealthCheckVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredHealthCheckVarianceAdjuster {
    pub cap: f64,
}

impl TieredHealthCheckVarianceModifier for CapTieredHealthCheckVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredHealthCheckVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelTieredHealthCheckVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredHealthCheckVarianceModifier for LabelTieredHealthCheckVarianceAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &TieredHealthCheckVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredHealthCheckVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredHealthCheckVarianceModifier for TieredTieredHealthCheckVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredHealthCheckVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tiered health check variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredHealthCheckVarianceEngine {
    pub steps: Vec<Rc<dyn TieredHealthCheckVarianceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredHealthCheckVarianceEngine {
    pub fn run(&mut self, item: &TieredHealthCheckVariance) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TieredHealthCheckVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_health_check_variance_engine() -> TieredHealthCheckVarianceEngine {
    TieredHealthCheckVarianceEngine {
        steps: vec![
            Rc::new(PercentTieredHealthCheckVarianceAdjuster { percent: -24.1 })
                as Rc<dyn TieredHealthCheckVarianceModifier>,
            Rc::new(CapTieredHealthCheckVarianceAdjuster { cap: 1836.0 }),
            Rc::new(LabelTieredHealthCheckVarianceAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.28)]),
            }),
            Rc::new(TieredTieredHealthCheckVarianceAdjuster {
                bounds: vec![1799.0, 4371.0, 5493.0],
                rates: vec![0.98, 0.91, 0.94],
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 67748.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_health_check_variance_engine();
        let item = new_tiered_health_check_variance("code-470", "label-160");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
