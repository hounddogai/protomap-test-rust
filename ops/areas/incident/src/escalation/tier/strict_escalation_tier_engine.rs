use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict escalation tier calculation.
pub trait StrictEscalationTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictEscalationTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictEscalationTierAdjuster {
    pub cap: f64,
}

impl StrictEscalationTierAdjuster for CapStrictEscalationTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictEscalationTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictEscalationTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictEscalationTierAdjuster for TieredStrictEscalationTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictEscalationTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationTierRevisionOffsetAdjuster {
    pub weight: f64,
}

impl StrictEscalationTierAdjuster for StrictEscalationTierRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &StrictEscalationTier, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictEscalationTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictEscalationTierAdjuster for FlaggedStrictEscalationTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictEscalationTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict escalation tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictEscalationTierEngine {
    pub steps: Vec<Rc<dyn StrictEscalationTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictEscalationTierEngine {
    pub fn compute(&mut self, item: &StrictEscalationTier) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[StrictEscalationTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_escalation_tier_engine(flags: &flags::Registry) -> StrictEscalationTierEngine {
    StrictEscalationTierEngine {
        steps: vec![
            Rc::new(CapStrictEscalationTierAdjuster { cap: 5836.0 }) as Rc<dyn StrictEscalationTierAdjuster>,
            Rc::new(TieredStrictEscalationTierAdjuster {
                bounds: vec![321.0, 2475.0, 2883.0],
                rates: vec![0.95, 0.83, 1.19],
            }),
            Rc::new(StrictEscalationTierRevisionOffsetAdjuster { weight: 0.52 }),
            Rc::new(FlaggedStrictEscalationTierAdjuster {
                flags: flags.clone(),
                flag: "strict-escalation-tier-adaptive".to_string(),
                percent: 0.4,
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 54779.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_escalation_tier_engine(&flags::new_registry());
        let item = new_strict_escalation_tier("code-298", "label-835");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
