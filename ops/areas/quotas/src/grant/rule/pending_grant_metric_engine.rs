use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending grant metric calculation.
pub trait PendingGrantMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingGrantMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingGrantMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingGrantMetricAdjuster for TieredPendingGrantMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingGrantMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryPendingGrantMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingGrantMetricAdjuster for CountryPendingGrantMetricAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &PendingGrantMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingGrantMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingGrantMetricAdjuster for FlaggedPendingGrantMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingGrantMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingGrantMetricAdjuster {
    pub percent: f64,
}

impl PendingGrantMetricAdjuster for PercentPendingGrantMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingGrantMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingGrantMetricSequenceOffsetAdjuster {
    pub weight: f64,
}

impl PendingGrantMetricAdjuster for PendingGrantMetricSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &PendingGrantMetric, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingGrantMetricAdjuster {
    pub step: f64,
}

impl PendingGrantMetricAdjuster for RoundingPendingGrantMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingGrantMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the pending grant metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingGrantMetricEngine {
    pub steps: Vec<Rc<dyn PendingGrantMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingGrantMetricEngine {
    pub fn evaluate(&mut self, item: &PendingGrantMetric) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PendingGrantMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_grant_metric_engine(flags: &flags::Registry) -> PendingGrantMetricEngine {
    PendingGrantMetricEngine {
        steps: vec![
            Rc::new(TieredPendingGrantMetricAdjuster {
                bounds: vec![2629.0, 6859.0, 9892.0],
                rates: vec![1.11, 1.14, 1.12],
            }) as Rc<dyn PendingGrantMetricAdjuster>,
            Rc::new(CountryPendingGrantMetricAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.37)]),
            }),
            Rc::new(FlaggedPendingGrantMetricAdjuster {
                flags: flags.clone(),
                flag: "pending-grant-metric-legacy".to_string(),
                percent: -9.5,
            }),
            Rc::new(PercentPendingGrantMetricAdjuster { percent: 1.6 }),
            Rc::new(PendingGrantMetricSequenceOffsetAdjuster { weight: 1.27 }),
            Rc::new(RoundingPendingGrantMetricAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 53043.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_grant_metric_engine(&flags::new_registry());
        let item = new_pending_grant_metric("code-594", "title-324");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
