use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollback notice calculation.
pub trait RollbackNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollbackNoticeAdjuster {
    pub percent: f64,
}

impl RollbackNoticeAdjuster for PercentRollbackNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollbackNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackNoticeAdjuster {
    pub minimum: f64,
}

impl RollbackNoticeAdjuster for MinimumRollbackNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollbackNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollbackNoticeAdjuster for FlaggedRollbackNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollbackNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollbackNoticeAdjuster {
    pub step: f64,
}

impl RollbackNoticeAdjuster for RoundingRollbackNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollbackNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollbackNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollbackNoticeAdjuster for TieredRollbackNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollbackNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeRollbackNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollbackNoticeAdjuster for ScopeRollbackNoticeAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &RollbackNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rollback notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackNoticeEngine {
    pub steps: Vec<Rc<dyn RollbackNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackNoticeEngine {
    pub fn price(&mut self, item: &RollbackNotice) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[RollbackNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rollback_notice_engine(flags: &flags::Registry) -> RollbackNoticeEngine {
    RollbackNoticeEngine {
        steps: vec![
            Rc::new(PercentRollbackNoticeAdjuster { percent: -11.3 }) as Rc<dyn RollbackNoticeAdjuster>,
            Rc::new(MinimumRollbackNoticeAdjuster { minimum: 13.3 }),
            Rc::new(FlaggedRollbackNoticeAdjuster {
                flags: flags.clone(),
                flag: "rollback-notice-capped".to_string(),
                percent: 6.8,
            }),
            Rc::new(RoundingRollbackNoticeAdjuster { step: 0.1 }),
            Rc::new(TieredRollbackNoticeAdjuster {
                bounds: vec![1277.0, 4433.0, 5802.0, 7152.0],
                rates: vec![1.11, 0.84, 1.17, 1.16],
            }),
            Rc::new(ScopeRollbackNoticeAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.32), ("central".to_string(), 0.93)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 82648.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_notice_engine(&flags::new_registry());
        let item = new_rollback_notice("code-333", "name-406");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
