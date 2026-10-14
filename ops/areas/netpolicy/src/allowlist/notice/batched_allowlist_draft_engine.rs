use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched allowlist draft calculation.
pub trait BatchedAllowlistDraftAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedAllowlistDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedAllowlistDraftAdjuster {
    pub step: f64,
}

impl BatchedAllowlistDraftAdjuster for RoundingBatchedAllowlistDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedAllowlistDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedAllowlistDraftAdjuster {
    pub minimum: f64,
}

impl BatchedAllowlistDraftAdjuster for MinimumBatchedAllowlistDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedAllowlistDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelBatchedAllowlistDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedAllowlistDraftAdjuster for ChannelBatchedAllowlistDraftAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &BatchedAllowlistDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistDraftWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl BatchedAllowlistDraftAdjuster for BatchedAllowlistDraftWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedAllowlistDraft, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedAllowlistDraftAdjuster {
    pub cap: f64,
}

impl BatchedAllowlistDraftAdjuster for CapBatchedAllowlistDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedAllowlistDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedAllowlistDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedAllowlistDraftAdjuster for TieredBatchedAllowlistDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedAllowlistDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the batched allowlist draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedAllowlistDraftEngine {
    pub steps: Vec<Rc<dyn BatchedAllowlistDraftAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedAllowlistDraftEngine {
    pub fn compute(&mut self, item: &BatchedAllowlistDraft) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedAllowlistDraftAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedAllowlistDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_allowlist_draft_engine() -> BatchedAllowlistDraftEngine {
    BatchedAllowlistDraftEngine {
        steps: vec![
            Rc::new(RoundingBatchedAllowlistDraftAdjuster { step: 0.05 }) as Rc<dyn BatchedAllowlistDraftAdjuster>,
            Rc::new(MinimumBatchedAllowlistDraftAdjuster { minimum: 16.1 }),
            Rc::new(ChannelBatchedAllowlistDraftAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.38),
                    ("online".to_string(), 0.99),
                    ("north".to_string(), 0.85),
                ]),
            }),
            Rc::new(BatchedAllowlistDraftWindowDaysOffsetAdjuster { weight: 1.37 }),
            Rc::new(CapBatchedAllowlistDraftAdjuster { cap: 49928.0 }),
            Rc::new(TieredBatchedAllowlistDraftAdjuster {
                bounds: vec![3658.0, 5418.0, 6056.0],
                rates: vec![1.02, 0.92, 1.07],
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 32255.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_allowlist_draft_engine();
        let item = new_batched_allowlist_draft("code-423", "name-605");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
