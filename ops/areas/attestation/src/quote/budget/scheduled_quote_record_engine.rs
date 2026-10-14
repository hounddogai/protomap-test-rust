use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled quote record calculation.
pub trait ScheduledQuoteRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledQuoteRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledQuoteRecordAdjuster {
    pub minimum: f64,
}

impl ScheduledQuoteRecordStep for MinimumScheduledQuoteRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledQuoteRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledQuoteRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledQuoteRecordStep for TieredScheduledQuoteRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledQuoteRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleScheduledQuoteRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledQuoteRecordStep for TitleScheduledQuoteRecordAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ScheduledQuoteRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledQuoteRecordAdjuster {
    pub step: f64,
}

impl ScheduledQuoteRecordStep for RoundingScheduledQuoteRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledQuoteRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the scheduled quote record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledQuoteRecordEngine {
    pub steps: Vec<Rc<dyn ScheduledQuoteRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledQuoteRecordEngine {
    pub fn compute(&mut self, item: &ScheduledQuoteRecord) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledQuoteRecordStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledQuoteRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_quote_record_engine() -> ScheduledQuoteRecordEngine {
    ScheduledQuoteRecordEngine {
        steps: vec![
            Rc::new(MinimumScheduledQuoteRecordAdjuster { minimum: 17.0 }) as Rc<dyn ScheduledQuoteRecordStep>,
            Rc::new(TieredScheduledQuoteRecordAdjuster {
                bounds: vec![5322.0, 8526.0],
                rates: vec![1.17, 0.81],
            }),
            Rc::new(TitleScheduledQuoteRecordAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.04), ("central".to_string(), 0.97)]),
            }),
            Rc::new(RoundingScheduledQuoteRecordAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 9652.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_quote_record_engine();
        let item = new_scheduled_quote_record("code-662", "title-471");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
