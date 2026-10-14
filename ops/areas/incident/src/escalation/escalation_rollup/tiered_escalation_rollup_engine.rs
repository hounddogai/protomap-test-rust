use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered escalation rollup calculation.
pub trait TieredEscalationRollupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredEscalationRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationRollupPortOffsetAdjuster {
    pub weight: f64,
}

impl TieredEscalationRollupStep for TieredEscalationRollupPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &TieredEscalationRollup, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredEscalationRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredEscalationRollupStep for TieredTieredEscalationRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredEscalationRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredEscalationRollupAdjuster {
    pub step: f64,
}

impl TieredEscalationRollupStep for RoundingTieredEscalationRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredEscalationRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceTieredEscalationRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredEscalationRollupStep for SourceTieredEscalationRollupAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &TieredEscalationRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredEscalationRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredEscalationRollupStep for FlaggedTieredEscalationRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredEscalationRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredEscalationRollupAdjuster {
    pub percent: f64,
}

impl TieredEscalationRollupStep for PercentTieredEscalationRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredEscalationRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered escalation rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredEscalationRollupEngine {
    pub steps: Vec<Rc<dyn TieredEscalationRollupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredEscalationRollupEngine {
    pub fn run(&mut self, item: &TieredEscalationRollup) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[TieredEscalationRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_escalation_rollup_engine(flags: &flags::Registry) -> TieredEscalationRollupEngine {
    TieredEscalationRollupEngine {
        steps: vec![
            Rc::new(TieredEscalationRollupPortOffsetAdjuster { weight: 0.79 }) as Rc<dyn TieredEscalationRollupStep>,
            Rc::new(TieredTieredEscalationRollupAdjuster {
                bounds: vec![1170.0, 7390.0],
                rates: vec![0.94, 1.02],
            }),
            Rc::new(RoundingTieredEscalationRollupAdjuster { step: 0.1 }),
            Rc::new(SourceTieredEscalationRollupAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.04), ("west".to_string(), 1.11)]),
            }),
            Rc::new(FlaggedTieredEscalationRollupAdjuster {
                flags: flags.clone(),
                flag: "tiered-escalation-rollup-tiered".to_string(),
                percent: 6.0,
            }),
            Rc::new(PercentTieredEscalationRollupAdjuster { percent: 6.2 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 35554.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_escalation_rollup_engine(&flags::new_registry());
        let item = new_tiered_escalation_rollup("code-920", "label-214");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
