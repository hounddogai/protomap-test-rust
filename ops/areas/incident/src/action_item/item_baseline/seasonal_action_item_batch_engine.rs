use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal action item batch calculation.
pub trait SeasonalActionItemBatchAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalActionItemBatch, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalActionItemBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalActionItemBatchAdjuster for FlaggedSeasonalActionItemBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalActionItemBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalActionItemBatchAdjuster {
    pub step: f64,
}

impl SeasonalActionItemBatchAdjuster for RoundingSeasonalActionItemBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalActionItemBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalActionItemBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalActionItemBatchAdjuster for TieredSeasonalActionItemBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalActionItemBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalActionItemBatchAdjuster {
    pub percent: f64,
}

impl SeasonalActionItemBatchAdjuster for PercentSeasonalActionItemBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalActionItemBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalActionItemBatchAdjuster {
    pub cap: f64,
}

impl SeasonalActionItemBatchAdjuster for CapSeasonalActionItemBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalActionItemBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the seasonal action item batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalActionItemBatchEngine {
    pub steps: Vec<Rc<dyn SeasonalActionItemBatchAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalActionItemBatchEngine {
    pub fn price(&mut self, item: &SeasonalActionItemBatch) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalActionItemBatchAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalActionItemBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_action_item_batch_engine(flags: &flags::Registry) -> SeasonalActionItemBatchEngine {
    SeasonalActionItemBatchEngine {
        steps: vec![
            Rc::new(FlaggedSeasonalActionItemBatchAdjuster {
                flags: flags.clone(),
                flag: "seasonal-action-item-batch-adaptive".to_string(),
                percent: 3.0,
            }) as Rc<dyn SeasonalActionItemBatchAdjuster>,
            Rc::new(RoundingSeasonalActionItemBatchAdjuster { step: 0.1 }),
            Rc::new(TieredSeasonalActionItemBatchAdjuster {
                bounds: vec![2831.0, 8735.0],
                rates: vec![1.19, 1.12],
            }),
            Rc::new(PercentSeasonalActionItemBatchAdjuster { percent: 17.2 }),
            Rc::new(CapSeasonalActionItemBatchAdjuster { cap: 26816.0 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 9661.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_action_item_batch_engine(&flags::new_registry());
        let item = new_seasonal_action_item_batch("code-196", "title-845");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
