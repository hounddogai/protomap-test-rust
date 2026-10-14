use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered postmortem cycle calculation.
pub trait TieredPostmortemCycleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredPostmortemCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredPostmortemCycleAdjuster {
    pub percent: f64,
}

impl TieredPostmortemCycleFactor for PercentTieredPostmortemCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredPostmortemCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredPostmortemCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredPostmortemCycleFactor for FlaggedTieredPostmortemCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredPostmortemCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredPostmortemCycleAdjuster {
    pub minimum: f64,
}

impl TieredPostmortemCycleFactor for MinimumTieredPostmortemCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredPostmortemCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentTieredPostmortemCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredPostmortemCycleFactor for SegmentTieredPostmortemCycleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &TieredPostmortemCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the tiered postmortem cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredPostmortemCycleEngine {
    pub steps: Vec<Rc<dyn TieredPostmortemCycleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredPostmortemCycleEngine {
    pub fn run(&mut self, item: &TieredPostmortemCycle) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TieredPostmortemCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_postmortem_cycle_engine(flags: &flags::Registry) -> TieredPostmortemCycleEngine {
    TieredPostmortemCycleEngine {
        steps: vec![
            Rc::new(PercentTieredPostmortemCycleAdjuster { percent: 6.4 }) as Rc<dyn TieredPostmortemCycleFactor>,
            Rc::new(FlaggedTieredPostmortemCycleAdjuster {
                flags: flags.clone(),
                flag: "tiered-postmortem-cycle-weighted".to_string(),
                percent: 6.1,
            }),
            Rc::new(MinimumTieredPostmortemCycleAdjuster { minimum: 36.7 }),
            Rc::new(SegmentTieredPostmortemCycleAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.91),
                    ("central".to_string(), 0.85),
                    ("north".to_string(), 1.08),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 99956.1,
    }
}
