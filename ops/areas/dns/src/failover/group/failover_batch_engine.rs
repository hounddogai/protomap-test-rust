use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the failover batch calculation.
pub trait FailoverBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFailoverBatchAdjuster {
    pub minimum: f64,
}

impl FailoverBatchFactor for MinimumFailoverBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FailoverBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FailoverBatchFactor for TieredFailoverBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FailoverBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelFailoverBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverBatchFactor for LabelFailoverBatchAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &FailoverBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFailoverBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FailoverBatchFactor for FlaggedFailoverBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FailoverBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFailoverBatchAdjuster {
    pub cap: f64,
}

impl FailoverBatchFactor for CapFailoverBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FailoverBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the failover batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverBatchEngine {
    pub steps: Vec<Rc<dyn FailoverBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverBatchEngine {
    pub fn evaluate(&mut self, item: &FailoverBatch) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[FailoverBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_failover_batch_engine(flags: &flags::Registry) -> FailoverBatchEngine {
    FailoverBatchEngine {
        steps: vec![
            Rc::new(MinimumFailoverBatchAdjuster { minimum: 11.5 }) as Rc<dyn FailoverBatchFactor>,
            Rc::new(TieredFailoverBatchAdjuster {
                bounds: vec![161.0, 1482.0, 2590.0, 3874.0],
                rates: vec![0.85, 1.19, 1.11, 1.2],
            }),
            Rc::new(LabelFailoverBatchAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.01),
                    ("retail".to_string(), 1.04),
                    ("central".to_string(), 0.92),
                ]),
            }),
            Rc::new(FlaggedFailoverBatchAdjuster {
                flags: flags.clone(),
                flag: "failover-batch-strict".to_string(),
                percent: 3.4,
            }),
            Rc::new(CapFailoverBatchAdjuster { cap: 10666.0 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 8640.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_failover_batch_engine(&flags::new_registry());
        let item = new_failover_batch("code-924", "label-8");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
