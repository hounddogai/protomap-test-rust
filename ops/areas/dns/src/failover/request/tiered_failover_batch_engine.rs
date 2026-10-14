use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the tiered failover batch calculation.
pub trait TieredFailoverBatchAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredFailoverBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelTieredFailoverBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredFailoverBatchAdjuster for LabelTieredFailoverBatchAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &TieredFailoverBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredFailoverBatchAdjuster {
    pub percent: f64,
}

impl TieredFailoverBatchAdjuster for PercentTieredFailoverBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredFailoverBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverBatchRateOffsetAdjuster {
    pub weight: f64,
}

impl TieredFailoverBatchAdjuster for TieredFailoverBatchRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &TieredFailoverBatch, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredFailoverBatchAdjuster {
    pub minimum: f64,
}

impl TieredFailoverBatchAdjuster for MinimumTieredFailoverBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredFailoverBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered failover batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredFailoverBatchEngine {
    pub steps: Vec<Rc<dyn TieredFailoverBatchAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredFailoverBatchEngine {
    pub fn evaluate(&mut self, item: &TieredFailoverBatch) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredFailoverBatchAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredFailoverBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_failover_batch_engine() -> TieredFailoverBatchEngine {
    TieredFailoverBatchEngine {
        steps: vec![
            Rc::new(LabelTieredFailoverBatchAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.85)]),
            }) as Rc<dyn TieredFailoverBatchAdjuster>,
            Rc::new(PercentTieredFailoverBatchAdjuster { percent: -1.6 }),
            Rc::new(TieredFailoverBatchRateOffsetAdjuster { weight: 1.45 }),
            Rc::new(MinimumTieredFailoverBatchAdjuster { minimum: 19.2 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 11293.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_failover_batch_engine();
        let item = new_tiered_failover_batch("code-169", "label-505");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
