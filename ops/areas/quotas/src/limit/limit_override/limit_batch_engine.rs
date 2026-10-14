use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the limit batch calculation.
pub trait LimitBatchAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LimitBatchAdjuster for TieredLimitBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LimitBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLimitBatchAdjuster {
    pub minimum: f64,
}

impl LimitBatchAdjuster for MinimumLimitBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LimitBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLimitBatchAdjuster {
    pub step: f64,
}

impl LimitBatchAdjuster for RoundingLimitBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LimitBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryLimitBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LimitBatchAdjuster for CountryLimitBatchAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &LimitBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitBatchUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl LimitBatchAdjuster for LimitBatchUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &LimitBatch, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

/// Runs the limit batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitBatchEngine {
    pub steps: Vec<Rc<dyn LimitBatchAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitBatchEngine {
    pub fn compute(&mut self, item: &LimitBatch) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[LimitBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_limit_batch_engine() -> LimitBatchEngine {
    LimitBatchEngine {
        steps: vec![
            Rc::new(TieredLimitBatchAdjuster {
                bounds: vec![4557.0, 6742.0, 9973.0],
                rates: vec![0.91, 0.86, 0.93],
            }) as Rc<dyn LimitBatchAdjuster>,
            Rc::new(MinimumLimitBatchAdjuster { minimum: 15.7 }),
            Rc::new(RoundingLimitBatchAdjuster { step: 0.5 }),
            Rc::new(CountryLimitBatchAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.22),
                    ("online".to_string(), 0.94),
                    ("north".to_string(), 1.14),
                ]),
            }),
            Rc::new(LimitBatchUpdatedAtOffsetAdjuster { weight: 1.51 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 2552.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_batch_engine();
        let item = new_limit_batch("code-399", "title-365");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
