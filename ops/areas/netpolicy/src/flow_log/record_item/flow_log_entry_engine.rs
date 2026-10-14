use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the flow log entry calculation.
pub trait FlowLogEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FlowLogEntryAdjuster for TieredFlowLogEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FlowLogEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceFlowLogEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FlowLogEntryAdjuster for SourceFlowLogEntryAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &FlowLogEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFlowLogEntryAdjuster {
    pub percent: f64,
}

impl FlowLogEntryAdjuster for PercentFlowLogEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FlowLogEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the flow log entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogEntryEngine {
    pub steps: Vec<Rc<dyn FlowLogEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogEntryEngine {
    pub fn run(&mut self, item: &FlowLogEntry) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FlowLogEntryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_flow_log_entry_engine() -> FlowLogEntryEngine {
    FlowLogEntryEngine {
        steps: vec![
            Rc::new(TieredFlowLogEntryAdjuster {
                bounds: vec![8012.0, 9045.0],
                rates: vec![0.93, 1.11],
            }) as Rc<dyn FlowLogEntryAdjuster>,
            Rc::new(SourceFlowLogEntryAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.29), ("retail".to_string(), 1.23)]),
            }),
            Rc::new(PercentFlowLogEntryAdjuster { percent: -16.4 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 93360.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_flow_log_entry_engine();
        let item = new_flow_log_entry("code-985", "title-604");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
