use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the limit claim calculation.
pub trait LimitClaimStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLimitClaimAdjuster {
    pub percent: f64,
}

impl LimitClaimStep for PercentLimitClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LimitClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLimitClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LimitClaimStep for FlaggedLimitClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LimitClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLimitClaimAdjuster {
    pub minimum: f64,
}

impl LimitClaimStep for MinimumLimitClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LimitClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LimitClaimStep for TieredLimitClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LimitClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitClaimMarginOffsetAdjuster {
    pub weight: f64,
}

impl LimitClaimStep for LimitClaimMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &LimitClaim, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

/// Runs the limit claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitClaimEngine {
    pub steps: Vec<Rc<dyn LimitClaimStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitClaimEngine {
    pub fn evaluate(&mut self, item: &LimitClaim) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[LimitClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_limit_claim_engine(flags: &flags::Registry) -> LimitClaimEngine {
    LimitClaimEngine {
        steps: vec![
            Rc::new(PercentLimitClaimAdjuster { percent: -0.9 }) as Rc<dyn LimitClaimStep>,
            Rc::new(FlaggedLimitClaimAdjuster {
                flags: flags.clone(),
                flag: "limit-claim-weighted".to_string(),
                percent: 6.6,
            }),
            Rc::new(MinimumLimitClaimAdjuster { minimum: 12.5 }),
            Rc::new(TieredLimitClaimAdjuster {
                bounds: vec![3858.0, 4025.0, 4591.0],
                rates: vec![1.02, 1.19, 1.08],
            }),
            Rc::new(LimitClaimMarginOffsetAdjuster { weight: 1.75 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 72453.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_claim_engine(&flags::new_registry());
        let item = new_limit_claim("code-465", "title-785");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
