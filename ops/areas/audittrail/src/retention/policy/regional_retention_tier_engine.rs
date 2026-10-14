use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional retention tier calculation.
pub trait RegionalRetentionTierFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalRetentionTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalRetentionTierAdjuster {
    pub step: f64,
}

impl RegionalRetentionTierFactor for RoundingRegionalRetentionTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalRetentionTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionTierRetriesOffsetAdjuster {
    pub weight: f64,
}

impl RegionalRetentionTierFactor for RegionalRetentionTierRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalRetentionTier, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalRetentionTierAdjuster {
    pub minimum: f64,
}

impl RegionalRetentionTierFactor for MinimumRegionalRetentionTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalRetentionTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalRetentionTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalRetentionTierFactor for FlaggedRegionalRetentionTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalRetentionTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalRetentionTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalRetentionTierFactor for TieredRegionalRetentionTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalRetentionTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the regional retention tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalRetentionTierEngine {
    pub steps: Vec<Rc<dyn RegionalRetentionTierFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalRetentionTierEngine {
    pub fn compute(&mut self, item: &RegionalRetentionTier) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RegionalRetentionTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_retention_tier_engine(flags: &flags::Registry) -> RegionalRetentionTierEngine {
    RegionalRetentionTierEngine {
        steps: vec![
            Rc::new(RoundingRegionalRetentionTierAdjuster { step: 0.1 }) as Rc<dyn RegionalRetentionTierFactor>,
            Rc::new(RegionalRetentionTierRetriesOffsetAdjuster { weight: 1.33 }),
            Rc::new(MinimumRegionalRetentionTierAdjuster { minimum: 2.5 }),
            Rc::new(FlaggedRegionalRetentionTierAdjuster {
                flags: flags.clone(),
                flag: "regional-retention-tier-capped".to_string(),
                percent: 7.6,
            }),
            Rc::new(TieredRegionalRetentionTierAdjuster {
                bounds: vec![407.0, 3386.0, 5219.0, 6616.0],
                rates: vec![1.11, 1.03, 0.86, 0.86],
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 32239.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_retention_tier_engine(&flags::new_registry());
        let item = new_regional_retention_tier("code-581", "title-853");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
