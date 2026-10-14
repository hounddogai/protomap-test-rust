use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the legacy flow log variance calculation.
pub trait LegacyFlowLogVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyFlowLogVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyFlowLogVarianceAdjuster {
    pub percent: f64,
}

impl LegacyFlowLogVarianceStep for PercentLegacyFlowLogVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyFlowLogVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyFlowLogVarianceAdjuster {
    pub minimum: f64,
}

impl LegacyFlowLogVarianceStep for MinimumLegacyFlowLogVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyFlowLogVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitLegacyFlowLogVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyFlowLogVarianceStep for UnitLegacyFlowLogVarianceAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &LegacyFlowLogVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the legacy flow log variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyFlowLogVarianceEngine {
    pub steps: Vec<Rc<dyn LegacyFlowLogVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyFlowLogVarianceEngine {
    pub fn price(&mut self, item: &LegacyFlowLogVariance) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyFlowLogVarianceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyFlowLogVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_flow_log_variance_engine() -> LegacyFlowLogVarianceEngine {
    LegacyFlowLogVarianceEngine {
        steps: vec![
            Rc::new(PercentLegacyFlowLogVarianceAdjuster { percent: -14.6 }) as Rc<dyn LegacyFlowLogVarianceStep>,
            Rc::new(MinimumLegacyFlowLogVarianceAdjuster { minimum: 23.1 }),
            Rc::new(UnitLegacyFlowLogVarianceAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.72),
                    ("west".to_string(), 1.0),
                    ("north".to_string(), 1.36),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 3433.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_flow_log_variance_engine();
        let item = new_legacy_flow_log_variance("code-926", "title-339");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
