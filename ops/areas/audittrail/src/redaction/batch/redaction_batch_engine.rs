use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the redaction batch calculation.
pub trait RedactionBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRedactionBatchAdjuster {
    pub step: f64,
}

impl RedactionBatchModifier for RoundingRedactionBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RedactionBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionBatchModifier for TieredRedactionBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBatchMinimumOffsetAdjuster {
    pub weight: f64,
}

impl RedactionBatchModifier for RedactionBatchMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &RedactionBatch, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRedactionBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RedactionBatchModifier for FlaggedRedactionBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RedactionBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRedactionBatchAdjuster {
    pub minimum: f64,
}

impl RedactionBatchModifier for MinimumRedactionBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RedactionBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleRedactionBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionBatchModifier for LocaleRedactionBatchAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &RedactionBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the redaction batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionBatchEngine {
    pub steps: Vec<Rc<dyn RedactionBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionBatchEngine {
    pub fn run(&mut self, item: &RedactionBatch) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RedactionBatchModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RedactionBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_redaction_batch_engine(flags: &flags::Registry) -> RedactionBatchEngine {
    RedactionBatchEngine {
        steps: vec![
            Rc::new(RoundingRedactionBatchAdjuster { step: 1.0 }) as Rc<dyn RedactionBatchModifier>,
            Rc::new(TieredRedactionBatchAdjuster {
                bounds: vec![4846.0, 5558.0, 6045.0],
                rates: vec![0.89, 0.98, 1.16],
            }),
            Rc::new(RedactionBatchMinimumOffsetAdjuster { weight: 0.63 }),
            Rc::new(FlaggedRedactionBatchAdjuster {
                flags: flags.clone(),
                flag: "redaction-batch-regional".to_string(),
                percent: -5.0,
            }),
            Rc::new(MinimumRedactionBatchAdjuster { minimum: 32.8 }),
            Rc::new(LocaleRedactionBatchAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.0),
                    ("north".to_string(), 1.0),
                    ("online".to_string(), 1.06),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 25319.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_batch_engine(&flags::new_registry());
        let item = new_redaction_batch("code-525", "label-70");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
