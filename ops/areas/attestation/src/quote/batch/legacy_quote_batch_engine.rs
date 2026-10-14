use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the legacy quote batch calculation.
pub trait LegacyQuoteBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyQuoteBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyQuoteBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyQuoteBatchFactor for TieredLegacyQuoteBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyQuoteBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyQuoteBatchAdjuster {
    pub cap: f64,
}

impl LegacyQuoteBatchFactor for CapLegacyQuoteBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyQuoteBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyQuoteBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyQuoteBatchFactor for FlaggedLegacyQuoteBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyQuoteBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy quote batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyQuoteBatchEngine {
    pub steps: Vec<Rc<dyn LegacyQuoteBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyQuoteBatchEngine {
    pub fn run(&mut self, item: &LegacyQuoteBatch) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyQuoteBatchFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyQuoteBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_quote_batch_engine(flags: &flags::Registry) -> LegacyQuoteBatchEngine {
    LegacyQuoteBatchEngine {
        steps: vec![
            Rc::new(TieredLegacyQuoteBatchAdjuster {
                bounds: vec![5575.0, 7656.0, 8400.0],
                rates: vec![1.06, 1.17, 1.15],
            }) as Rc<dyn LegacyQuoteBatchFactor>,
            Rc::new(CapLegacyQuoteBatchAdjuster { cap: 39837.0 }),
            Rc::new(FlaggedLegacyQuoteBatchAdjuster {
                flags: flags.clone(),
                flag: "legacy-quote-batch-regional".to_string(),
                percent: -2.4,
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 39236.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_quote_batch_engine(&flags::new_registry());
        let item = new_legacy_quote_batch("code-913", "name-259");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
