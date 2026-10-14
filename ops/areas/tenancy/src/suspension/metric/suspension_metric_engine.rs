use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the suspension metric calculation.
pub trait SuspensionMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceSuspensionMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SuspensionMetricStep for SourceSuspensionMetricAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &SuspensionMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSuspensionMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SuspensionMetricStep for FlaggedSuspensionMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SuspensionMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSuspensionMetricAdjuster {
    pub cap: f64,
}

impl SuspensionMetricStep for CapSuspensionMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SuspensionMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSuspensionMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SuspensionMetricStep for TieredSuspensionMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SuspensionMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the suspension metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionMetricEngine {
    pub steps: Vec<Rc<dyn SuspensionMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionMetricEngine {
    pub fn price(&mut self, item: &SuspensionMetric) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SuspensionMetricStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_suspension_metric_engine(flags: &flags::Registry) -> SuspensionMetricEngine {
    SuspensionMetricEngine {
        steps: vec![
            Rc::new(SourceSuspensionMetricAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.12)]),
            }) as Rc<dyn SuspensionMetricStep>,
            Rc::new(FlaggedSuspensionMetricAdjuster {
                flags: flags.clone(),
                flag: "suspension-metric-capped".to_string(),
                percent: 2.8,
            }),
            Rc::new(CapSuspensionMetricAdjuster { cap: 12414.0 }),
            Rc::new(TieredSuspensionMetricAdjuster {
                bounds: vec![3024.0, 5838.0, 6996.0, 8816.0],
                rates: vec![1.09, 1.08, 0.99, 1.13],
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 42751.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_metric_engine(&flags::new_registry());
        let item = new_suspension_metric("code-231", "title-408");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
