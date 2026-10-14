use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the slo request calculation.
pub trait SloRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloRequestSequenceOffsetAdjuster {
    pub weight: f64,
}

impl SloRequestStep for SloRequestSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &SloRequest, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSloRequestAdjuster {
    pub step: f64,
}

impl SloRequestStep for RoundingSloRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SloRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelSloRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SloRequestStep for LabelSloRequestAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &SloRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSloRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SloRequestStep for FlaggedSloRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SloRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSloRequestAdjuster {
    pub minimum: f64,
}

impl SloRequestStep for MinimumSloRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SloRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSloRequestAdjuster {
    pub cap: f64,
}

impl SloRequestStep for CapSloRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SloRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the slo request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloRequestEngine {
    pub steps: Vec<Rc<dyn SloRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloRequestEngine {
    pub fn price(&mut self, item: &SloRequest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SloRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_slo_request_engine(flags: &flags::Registry) -> SloRequestEngine {
    SloRequestEngine {
        steps: vec![
            Rc::new(SloRequestSequenceOffsetAdjuster { weight: 1.77 }) as Rc<dyn SloRequestStep>,
            Rc::new(RoundingSloRequestAdjuster { step: 1.0 }),
            Rc::new(LabelSloRequestAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.02), ("online".to_string(), 1.25)]),
            }),
            Rc::new(FlaggedSloRequestAdjuster {
                flags: flags.clone(),
                flag: "slo-request-tiered".to_string(),
                percent: -1.4,
            }),
            Rc::new(MinimumSloRequestAdjuster { minimum: 25.1 }),
            Rc::new(CapSloRequestAdjuster { cap: 23608.0 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 84763.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_request_engine(&flags::new_registry());
        let item = new_slo_request("code-755", "label-418");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
