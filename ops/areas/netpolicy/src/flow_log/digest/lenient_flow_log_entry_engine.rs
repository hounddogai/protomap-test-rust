use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient flow log entry calculation.
pub trait LenientFlowLogEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientFlowLogEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientFlowLogEntryAdjuster {
    pub percent: f64,
}

impl LenientFlowLogEntryAdjuster for PercentLenientFlowLogEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientFlowLogEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientFlowLogEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientFlowLogEntryAdjuster for FlaggedLenientFlowLogEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientFlowLogEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLenientFlowLogEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientFlowLogEntryAdjuster for LabelLenientFlowLogEntryAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LenientFlowLogEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientFlowLogEntryAdjuster {
    pub cap: f64,
}

impl LenientFlowLogEntryAdjuster for CapLenientFlowLogEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientFlowLogEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the lenient flow log entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientFlowLogEntryEngine {
    pub steps: Vec<Rc<dyn LenientFlowLogEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientFlowLogEntryEngine {
    pub fn evaluate(&mut self, item: &LenientFlowLogEntry) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientFlowLogEntryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientFlowLogEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_flow_log_entry_engine(flags: &flags::Registry) -> LenientFlowLogEntryEngine {
    LenientFlowLogEntryEngine {
        steps: vec![
            Rc::new(PercentLenientFlowLogEntryAdjuster { percent: 17.3 }) as Rc<dyn LenientFlowLogEntryAdjuster>,
            Rc::new(FlaggedLenientFlowLogEntryAdjuster {
                flags: flags.clone(),
                flag: "lenient-flow-log-entry-lenient".to_string(),
                percent: 4.9,
            }),
            Rc::new(LabelLenientFlowLogEntryAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.91), ("retail".to_string(), 1.2)]),
            }),
            Rc::new(CapLenientFlowLogEntryAdjuster { cap: 15886.0 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 67574.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_flow_log_entry_engine(&flags::new_registry());
        let item = new_lenient_flow_log_entry("code-380", "label-231");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
