use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the segment rule calculation.
pub trait SegmentRuleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSegmentRuleAdjuster {
    pub step: f64,
}

impl SegmentRuleFactor for RoundingSegmentRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SegmentRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSegmentRuleAdjuster {
    pub percent: f64,
}

impl SegmentRuleFactor for PercentSegmentRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SegmentRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSegmentRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SegmentRuleFactor for FlaggedSegmentRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SegmentRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SegmentRuleFactor for TieredSegmentRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SegmentRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSegmentRuleAdjuster {
    pub cap: f64,
}

impl SegmentRuleFactor for CapSegmentRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SegmentRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRuleMinimumOffsetAdjuster {
    pub weight: f64,
}

impl SegmentRuleFactor for SegmentRuleMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &SegmentRule, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

/// Runs the segment rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentRuleEngine {
    pub steps: Vec<Rc<dyn SegmentRuleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentRuleEngine {
    pub fn price(&mut self, item: &SegmentRule) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SegmentRuleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SegmentRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_segment_rule_engine(flags: &flags::Registry) -> SegmentRuleEngine {
    SegmentRuleEngine {
        steps: vec![
            Rc::new(RoundingSegmentRuleAdjuster { step: 1.0 }) as Rc<dyn SegmentRuleFactor>,
            Rc::new(PercentSegmentRuleAdjuster { percent: -0.6 }),
            Rc::new(FlaggedSegmentRuleAdjuster {
                flags: flags.clone(),
                flag: "segment-rule-capped".to_string(),
                percent: 9.9,
            }),
            Rc::new(TieredSegmentRuleAdjuster {
                bounds: vec![1109.0, 2026.0, 8624.0],
                rates: vec![0.92, 0.8, 0.83],
            }),
            Rc::new(CapSegmentRuleAdjuster { cap: 9596.0 }),
            Rc::new(SegmentRuleMinimumOffsetAdjuster { weight: 1.57 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 98501.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_rule_engine(&flags::new_registry());
        let item = new_segment_rule("code-82", "label-570");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
