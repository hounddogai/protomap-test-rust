use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal freeze checkpoint calculation.
pub trait SeasonalFreezeCheckpointFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalFreezeCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalFreezeCheckpointAdjuster {
    pub step: f64,
}

impl SeasonalFreezeCheckpointFactor for RoundingSeasonalFreezeCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalFreezeCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalFreezeCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalFreezeCheckpointFactor for FlaggedSeasonalFreezeCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalFreezeCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalFreezeCheckpointAdjuster {
    pub minimum: f64,
}

impl SeasonalFreezeCheckpointFactor for MinimumSeasonalFreezeCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalFreezeCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelSeasonalFreezeCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalFreezeCheckpointFactor for LabelSeasonalFreezeCheckpointAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &SeasonalFreezeCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the seasonal freeze checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalFreezeCheckpointEngine {
    pub steps: Vec<Rc<dyn SeasonalFreezeCheckpointFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalFreezeCheckpointEngine {
    pub fn price(&mut self, item: &SeasonalFreezeCheckpoint) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalFreezeCheckpointFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalFreezeCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_freeze_checkpoint_engine(flags: &flags::Registry) -> SeasonalFreezeCheckpointEngine {
    SeasonalFreezeCheckpointEngine {
        steps: vec![
            Rc::new(RoundingSeasonalFreezeCheckpointAdjuster { step: 5.0 }) as Rc<dyn SeasonalFreezeCheckpointFactor>,
            Rc::new(FlaggedSeasonalFreezeCheckpointAdjuster {
                flags: flags.clone(),
                flag: "seasonal-freeze-checkpoint-adaptive".to_string(),
                percent: -8.1,
            }),
            Rc::new(MinimumSeasonalFreezeCheckpointAdjuster { minimum: 39.2 }),
            Rc::new(LabelSeasonalFreezeCheckpointAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.22), ("west".to_string(), 0.76)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 83852.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_freeze_checkpoint_engine(&flags::new_registry());
        let item = new_seasonal_freeze_checkpoint("code-13", "label-548");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
