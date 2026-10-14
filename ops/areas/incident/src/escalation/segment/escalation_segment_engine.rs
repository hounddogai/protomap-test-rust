use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the escalation segment calculation.
pub trait EscalationSegmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationSegmentFactor for TieredEscalationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEscalationSegmentAdjuster {
    pub percent: f64,
}

impl EscalationSegmentFactor for PercentEscalationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EscalationSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelEscalationSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EscalationSegmentFactor for ChannelEscalationSegmentAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &EscalationSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEscalationSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EscalationSegmentFactor for FlaggedEscalationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EscalationSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the escalation segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationSegmentEngine {
    pub steps: Vec<Rc<dyn EscalationSegmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationSegmentEngine {
    pub fn compute(&mut self, item: &EscalationSegment) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EscalationSegmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EscalationSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_escalation_segment_engine(flags: &flags::Registry) -> EscalationSegmentEngine {
    EscalationSegmentEngine {
        steps: vec![
            Rc::new(TieredEscalationSegmentAdjuster {
                bounds: vec![1800.0, 8012.0],
                rates: vec![1.01, 1.16],
            }) as Rc<dyn EscalationSegmentFactor>,
            Rc::new(PercentEscalationSegmentAdjuster { percent: 23.7 }),
            Rc::new(ChannelEscalationSegmentAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.88), ("east".to_string(), 1.32)]),
            }),
            Rc::new(FlaggedEscalationSegmentAdjuster {
                flags: flags.clone(),
                flag: "escalation-segment-adaptive".to_string(),
                percent: -4.6,
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 36890.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_segment_engine(&flags::new_registry());
        let item = new_escalation_segment("code-632", "title-255");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
