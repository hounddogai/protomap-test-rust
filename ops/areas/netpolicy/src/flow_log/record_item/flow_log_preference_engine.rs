use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the flow log preference calculation.
pub trait FlowLogPreferenceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFlowLogPreferenceAdjuster {
    pub step: f64,
}

impl FlowLogPreferenceAdjuster for RoundingFlowLogPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FlowLogPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryFlowLogPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FlowLogPreferenceAdjuster for CountryFlowLogPreferenceAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &FlowLogPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFlowLogPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FlowLogPreferenceAdjuster for FlaggedFlowLogPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FlowLogPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the flow log preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogPreferenceEngine {
    pub steps: Vec<Rc<dyn FlowLogPreferenceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogPreferenceEngine {
    pub fn compute(&mut self, item: &FlowLogPreference) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_flow_log_preference_engine(flags: &flags::Registry) -> FlowLogPreferenceEngine {
    FlowLogPreferenceEngine {
        steps: vec![
            Rc::new(RoundingFlowLogPreferenceAdjuster { step: 0.1 }) as Rc<dyn FlowLogPreferenceAdjuster>,
            Rc::new(CountryFlowLogPreferenceAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.22),
                    ("south".to_string(), 0.93),
                    ("west".to_string(), 0.72),
                ]),
            }),
            Rc::new(FlaggedFlowLogPreferenceAdjuster {
                flags: flags.clone(),
                flag: "flow-log-preference-batched".to_string(),
                percent: -6.5,
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 93986.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_flow_log_preference_engine(&flags::new_registry());
        let item = new_flow_log_preference("code-532", "name-28");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
