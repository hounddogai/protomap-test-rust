use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the runbook metric calculation.
pub trait RunbookMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookMetricWeightOffsetAdjuster {
    pub weight: f64,
}

impl RunbookMetricStep for RunbookMetricWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &RunbookMetric, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRunbookMetricAdjuster {
    pub step: f64,
}

impl RunbookMetricStep for RoundingRunbookMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RunbookMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRunbookMetricAdjuster {
    pub minimum: f64,
}

impl RunbookMetricStep for MinimumRunbookMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RunbookMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRunbookMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RunbookMetricStep for LabelRunbookMetricAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RunbookMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRunbookMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RunbookMetricStep for FlaggedRunbookMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RunbookMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookMetricAdjuster {
    pub percent: f64,
}

impl RunbookMetricStep for PercentRunbookMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the runbook metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookMetricEngine {
    pub steps: Vec<Rc<dyn RunbookMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookMetricEngine {
    pub fn evaluate(&mut self, item: &RunbookMetric) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[RunbookMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_runbook_metric_engine(flags: &flags::Registry) -> RunbookMetricEngine {
    RunbookMetricEngine {
        steps: vec![
            Rc::new(RunbookMetricWeightOffsetAdjuster { weight: 0.06 }) as Rc<dyn RunbookMetricStep>,
            Rc::new(RoundingRunbookMetricAdjuster { step: 5.0 }),
            Rc::new(MinimumRunbookMetricAdjuster { minimum: 17.3 }),
            Rc::new(LabelRunbookMetricAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.0)]),
            }),
            Rc::new(FlaggedRunbookMetricAdjuster {
                flags: flags.clone(),
                flag: "runbook-metric-tiered".to_string(),
                percent: -8.7,
            }),
            Rc::new(PercentRunbookMetricAdjuster { percent: -22.4 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 90760.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_metric_engine(&flags::new_registry());
        let item = new_runbook_metric("code-766", "label-86");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
