use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal canary threshold calculation.
pub trait SeasonalCanaryThresholdAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalCanaryThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryThresholdQuantityOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalCanaryThresholdAdjuster for SeasonalCanaryThresholdQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalCanaryThreshold, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalCanaryThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalCanaryThresholdAdjuster for TieredSeasonalCanaryThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalCanaryThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalCanaryThresholdAdjuster {
    pub step: f64,
}

impl SeasonalCanaryThresholdAdjuster for RoundingSeasonalCanaryThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalCanaryThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelSeasonalCanaryThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalCanaryThresholdAdjuster for ChannelSeasonalCanaryThresholdAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &SeasonalCanaryThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalCanaryThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalCanaryThresholdAdjuster for FlaggedSeasonalCanaryThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalCanaryThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal canary threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalCanaryThresholdEngine {
    pub steps: Vec<Rc<dyn SeasonalCanaryThresholdAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalCanaryThresholdEngine {
    pub fn compute(&mut self, item: &SeasonalCanaryThreshold) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalCanaryThresholdAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalCanaryThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_seasonal_canary_threshold_engine(flags: &flags::Registry) -> SeasonalCanaryThresholdEngine {
    SeasonalCanaryThresholdEngine {
        steps: vec![
            Rc::new(SeasonalCanaryThresholdQuantityOffsetAdjuster { weight: 1.68 })
                as Rc<dyn SeasonalCanaryThresholdAdjuster>,
            Rc::new(TieredSeasonalCanaryThresholdAdjuster {
                bounds: vec![9094.0, 9998.0],
                rates: vec![1.16, 0.87],
            }),
            Rc::new(RoundingSeasonalCanaryThresholdAdjuster { step: 0.1 }),
            Rc::new(ChannelSeasonalCanaryThresholdAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.84)]),
            }),
            Rc::new(FlaggedSeasonalCanaryThresholdAdjuster {
                flags: flags.clone(),
                flag: "seasonal-canary-threshold-rolling".to_string(),
                percent: 8.3,
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 82773.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_canary_threshold_engine(&flags::new_registry());
        let item = new_seasonal_canary_threshold("code-326", "name-636");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
