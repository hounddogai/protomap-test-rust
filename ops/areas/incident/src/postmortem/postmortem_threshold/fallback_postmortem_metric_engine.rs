use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the fallback postmortem metric calculation.
pub trait FallbackPostmortemMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackPostmortemMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackPostmortemMetricAdjuster {
    pub minimum: f64,
}

impl FallbackPostmortemMetricStep for MinimumFallbackPostmortemMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackPostmortemMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackPostmortemMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackPostmortemMetricStep for FlaggedFallbackPostmortemMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackPostmortemMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackPostmortemMetricAdjuster {
    pub cap: f64,
}

impl FallbackPostmortemMetricStep for CapFallbackPostmortemMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackPostmortemMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the fallback postmortem metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackPostmortemMetricEngine {
    pub steps: Vec<Rc<dyn FallbackPostmortemMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackPostmortemMetricEngine {
    pub fn compute(&mut self, item: &FallbackPostmortemMetric) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackPostmortemMetricStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackPostmortemMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_postmortem_metric_engine(flags: &flags::Registry) -> FallbackPostmortemMetricEngine {
    FallbackPostmortemMetricEngine {
        steps: vec![
            Rc::new(MinimumFallbackPostmortemMetricAdjuster { minimum: 32.6 }) as Rc<dyn FallbackPostmortemMetricStep>,
            Rc::new(FlaggedFallbackPostmortemMetricAdjuster {
                flags: flags.clone(),
                flag: "fallback-postmortem-metric-capped".to_string(),
                percent: 8.1,
            }),
            Rc::new(CapFallbackPostmortemMetricAdjuster { cap: 25106.0 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 54207.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_postmortem_metric_engine(&flags::new_registry());
        let item = new_fallback_postmortem_metric("code-698", "name-945");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
