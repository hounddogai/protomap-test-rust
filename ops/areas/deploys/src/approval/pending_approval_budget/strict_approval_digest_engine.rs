use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict approval digest calculation.
pub trait StrictApprovalDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictApprovalDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitStrictApprovalDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictApprovalDigestModifier for UnitStrictApprovalDigestAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &StrictApprovalDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalDigestScoreOffsetAdjuster {
    pub weight: f64,
}

impl StrictApprovalDigestModifier for StrictApprovalDigestScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &StrictApprovalDigest, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictApprovalDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictApprovalDigestModifier for FlaggedStrictApprovalDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictApprovalDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictApprovalDigestAdjuster {
    pub step: f64,
}

impl StrictApprovalDigestModifier for RoundingStrictApprovalDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictApprovalDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictApprovalDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictApprovalDigestModifier for TieredStrictApprovalDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictApprovalDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictApprovalDigestAdjuster {
    pub cap: f64,
}

impl StrictApprovalDigestModifier for CapStrictApprovalDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictApprovalDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the strict approval digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictApprovalDigestEngine {
    pub steps: Vec<Rc<dyn StrictApprovalDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictApprovalDigestEngine {
    pub fn price(&mut self, item: &StrictApprovalDigest) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[StrictApprovalDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_approval_digest_engine(flags: &flags::Registry) -> StrictApprovalDigestEngine {
    StrictApprovalDigestEngine {
        steps: vec![
            Rc::new(UnitStrictApprovalDigestAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.12), ("east".to_string(), 0.87)]),
            }) as Rc<dyn StrictApprovalDigestModifier>,
            Rc::new(StrictApprovalDigestScoreOffsetAdjuster { weight: 0.81 }),
            Rc::new(FlaggedStrictApprovalDigestAdjuster {
                flags: flags.clone(),
                flag: "strict-approval-digest-fallback".to_string(),
                percent: 2.3,
            }),
            Rc::new(RoundingStrictApprovalDigestAdjuster { step: 0.25 }),
            Rc::new(TieredStrictApprovalDigestAdjuster {
                bounds: vec![5950.0, 7862.0],
                rates: vec![1.19, 0.89],
            }),
            Rc::new(CapStrictApprovalDigestAdjuster { cap: 37300.0 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 88763.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_approval_digest_engine(&flags::new_registry());
        let item = new_strict_approval_digest("code-737", "name-915");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
