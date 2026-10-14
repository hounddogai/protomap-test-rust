use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional escalation adjustment calculation.
pub trait RegionalEscalationAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalEscalationAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalEscalationAdjustmentAdjuster {
    pub percent: f64,
}

impl RegionalEscalationAdjustmentModifier for PercentRegionalEscalationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalEscalationAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalEscalationAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalEscalationAdjustmentModifier for FlaggedRegionalEscalationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalEscalationAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRegionalEscalationAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalEscalationAdjustmentModifier for SegmentRegionalEscalationAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &RegionalEscalationAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEscalationAdjustmentThresholdOffsetAdjuster {
    pub weight: f64,
}

impl RegionalEscalationAdjustmentModifier for RegionalEscalationAdjustmentThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalEscalationAdjustment, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalEscalationAdjustmentAdjuster {
    pub minimum: f64,
}

impl RegionalEscalationAdjustmentModifier for MinimumRegionalEscalationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalEscalationAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the regional escalation adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalEscalationAdjustmentEngine {
    pub steps: Vec<Rc<dyn RegionalEscalationAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalEscalationAdjustmentEngine {
    pub fn price(&mut self, item: &RegionalEscalationAdjustment) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[RegionalEscalationAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_escalation_adjustment_engine(flags: &flags::Registry) -> RegionalEscalationAdjustmentEngine {
    RegionalEscalationAdjustmentEngine {
        steps: vec![
            Rc::new(PercentRegionalEscalationAdjustmentAdjuster { percent: -4.1 })
                as Rc<dyn RegionalEscalationAdjustmentModifier>,
            Rc::new(FlaggedRegionalEscalationAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "regional-escalation-adjustment-scheduled".to_string(),
                percent: -2.5,
            }),
            Rc::new(SegmentRegionalEscalationAdjustmentAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.08), ("east".to_string(), 1.2)]),
            }),
            Rc::new(RegionalEscalationAdjustmentThresholdOffsetAdjuster { weight: 1.98 }),
            Rc::new(MinimumRegionalEscalationAdjustmentAdjuster { minimum: 44.8 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 95142.4,
    }
}
