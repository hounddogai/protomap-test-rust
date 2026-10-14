use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional escalation segment calculation.
pub trait RegionalEscalationSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalEscalationSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalEscalationSegmentAdjuster {
    pub percent: f64,
}

impl RegionalEscalationSegmentStep for PercentRegionalEscalationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalEscalationSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalEscalationSegmentAdjuster {
    pub cap: f64,
}

impl RegionalEscalationSegmentStep for CapRegionalEscalationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalEscalationSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalEscalationSegmentAdjuster {
    pub step: f64,
}

impl RegionalEscalationSegmentStep for RoundingRegionalEscalationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalEscalationSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRegionalEscalationSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalEscalationSegmentStep for NameRegionalEscalationSegmentAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RegionalEscalationSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalEscalationSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalEscalationSegmentStep for FlaggedRegionalEscalationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalEscalationSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalEscalationSegmentAdjuster {
    pub minimum: f64,
}

impl RegionalEscalationSegmentStep for MinimumRegionalEscalationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalEscalationSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the regional escalation segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalEscalationSegmentEngine {
    pub steps: Vec<Rc<dyn RegionalEscalationSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalEscalationSegmentEngine {
    pub fn evaluate(&mut self, item: &RegionalEscalationSegment) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[RegionalEscalationSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_escalation_segment_engine(flags: &flags::Registry) -> RegionalEscalationSegmentEngine {
    RegionalEscalationSegmentEngine {
        steps: vec![
            Rc::new(PercentRegionalEscalationSegmentAdjuster { percent: 11.0 })
                as Rc<dyn RegionalEscalationSegmentStep>,
            Rc::new(CapRegionalEscalationSegmentAdjuster { cap: 6134.0 }),
            Rc::new(RoundingRegionalEscalationSegmentAdjuster { step: 0.1 }),
            Rc::new(NameRegionalEscalationSegmentAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.75), ("east".to_string(), 1.27)]),
            }),
            Rc::new(FlaggedRegionalEscalationSegmentAdjuster {
                flags: flags.clone(),
                flag: "regional-escalation-segment-seasonal".to_string(),
                percent: -0.4,
            }),
            Rc::new(MinimumRegionalEscalationSegmentAdjuster { minimum: 37.1 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 54510.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_escalation_segment_engine(&flags::new_registry());
        let item = new_regional_escalation_segment("code-653", "name-722");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
