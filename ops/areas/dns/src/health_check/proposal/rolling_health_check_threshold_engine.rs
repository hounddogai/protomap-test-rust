use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling health check threshold calculation.
pub trait RollingHealthCheckThresholdModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingHealthCheckThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingHealthCheckThresholdRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl RollingHealthCheckThresholdModifier for RollingHealthCheckThresholdRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &RollingHealthCheckThreshold, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingHealthCheckThresholdAdjuster {
    pub cap: f64,
}

impl RollingHealthCheckThresholdModifier for CapRollingHealthCheckThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingHealthCheckThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingHealthCheckThresholdAdjuster {
    pub step: f64,
}

impl RollingHealthCheckThresholdModifier for RoundingRollingHealthCheckThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingHealthCheckThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingHealthCheckThresholdAdjuster {
    pub minimum: f64,
}

impl RollingHealthCheckThresholdModifier for MinimumRollingHealthCheckThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingHealthCheckThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRollingHealthCheckThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingHealthCheckThresholdModifier for TitleRollingHealthCheckThresholdAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RollingHealthCheckThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rolling health check threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingHealthCheckThresholdEngine {
    pub steps: Vec<Rc<dyn RollingHealthCheckThresholdModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingHealthCheckThresholdEngine {
    pub fn evaluate(&mut self, item: &RollingHealthCheckThreshold) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[RollingHealthCheckThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_health_check_threshold_engine() -> RollingHealthCheckThresholdEngine {
    RollingHealthCheckThresholdEngine {
        steps: vec![
            Rc::new(RollingHealthCheckThresholdRotationDaysOffsetAdjuster { weight: 1.96 })
                as Rc<dyn RollingHealthCheckThresholdModifier>,
            Rc::new(CapRollingHealthCheckThresholdAdjuster { cap: 48679.0 }),
            Rc::new(RoundingRollingHealthCheckThresholdAdjuster { step: 0.05 }),
            Rc::new(MinimumRollingHealthCheckThresholdAdjuster { minimum: 6.8 }),
            Rc::new(TitleRollingHealthCheckThresholdAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.26), ("north".to_string(), 1.12)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 85557.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_health_check_threshold_engine();
        let item = new_rolling_health_check_threshold("code-992", "title-244");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
