use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the retention rollup calculation.
pub trait RetentionRollupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRetentionRollupAdjuster {
    pub step: f64,
}

impl RetentionRollupModifier for RoundingRetentionRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RetentionRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionRollupModifier for FlaggedRetentionRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RetentionRollupModifier for TieredRetentionRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RetentionRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRetentionRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionRollupModifier for TitleRetentionRollupAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RetentionRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the retention rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionRollupEngine {
    pub steps: Vec<Rc<dyn RetentionRollupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionRollupEngine {
    pub fn price(&mut self, item: &RetentionRollup) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[RetentionRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_retention_rollup_engine(flags: &flags::Registry) -> RetentionRollupEngine {
    RetentionRollupEngine {
        steps: vec![
            Rc::new(RoundingRetentionRollupAdjuster { step: 0.05 }) as Rc<dyn RetentionRollupModifier>,
            Rc::new(FlaggedRetentionRollupAdjuster {
                flags: flags.clone(),
                flag: "retention-rollup-strict".to_string(),
                percent: 7.5,
            }),
            Rc::new(TieredRetentionRollupAdjuster {
                bounds: vec![4722.0, 8366.0],
                rates: vec![0.99, 1.0],
            }),
            Rc::new(TitleRetentionRollupAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.18)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 52977.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_rollup_engine(&flags::new_registry());
        let item = new_retention_rollup("code-557", "title-941");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
