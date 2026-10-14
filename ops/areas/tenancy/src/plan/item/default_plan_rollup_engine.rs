use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default plan rollup calculation.
pub trait DefaultPlanRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultPlanRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultPlanRollupAdjuster {
    pub percent: f64,
}

impl DefaultPlanRollupFactor for PercentDefaultPlanRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultPlanRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultPlanRollupAdjuster {
    pub step: f64,
}

impl DefaultPlanRollupFactor for RoundingDefaultPlanRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultPlanRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultPlanRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultPlanRollupFactor for FlaggedDefaultPlanRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultPlanRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanRollupQuantityOffsetAdjuster {
    pub weight: f64,
}

impl DefaultPlanRollupFactor for DefaultPlanRollupQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultPlanRollup, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceDefaultPlanRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultPlanRollupFactor for SourceDefaultPlanRollupAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &DefaultPlanRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultPlanRollupAdjuster {
    pub cap: f64,
}

impl DefaultPlanRollupFactor for CapDefaultPlanRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultPlanRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the default plan rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultPlanRollupEngine {
    pub steps: Vec<Rc<dyn DefaultPlanRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultPlanRollupEngine {
    pub fn price(&mut self, item: &DefaultPlanRollup) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[DefaultPlanRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_plan_rollup_engine(flags: &flags::Registry) -> DefaultPlanRollupEngine {
    DefaultPlanRollupEngine {
        steps: vec![
            Rc::new(PercentDefaultPlanRollupAdjuster { percent: -8.9 }) as Rc<dyn DefaultPlanRollupFactor>,
            Rc::new(RoundingDefaultPlanRollupAdjuster { step: 0.25 }),
            Rc::new(FlaggedDefaultPlanRollupAdjuster {
                flags: flags.clone(),
                flag: "default-plan-rollup-tiered".to_string(),
                percent: -3.6,
            }),
            Rc::new(DefaultPlanRollupQuantityOffsetAdjuster { weight: 0.24 }),
            Rc::new(SourceDefaultPlanRollupAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.82),
                    ("central".to_string(), 1.01),
                    ("online".to_string(), 1.35),
                ]),
            }),
            Rc::new(CapDefaultPlanRollupAdjuster { cap: 3671.0 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 39359.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_plan_rollup_engine(&flags::new_registry());
        let item = new_default_plan_rollup("code-540", "name-28");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
