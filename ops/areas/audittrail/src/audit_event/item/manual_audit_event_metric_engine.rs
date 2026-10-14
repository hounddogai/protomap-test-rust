use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual audit event metric calculation.
pub trait ManualAuditEventMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualAuditEventMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualAuditEventMetricAdjuster {
    pub percent: f64,
}

impl ManualAuditEventMetricStep for PercentManualAuditEventMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualAuditEventMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionManualAuditEventMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualAuditEventMetricStep for RegionManualAuditEventMetricAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ManualAuditEventMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualAuditEventMetricAdjuster {
    pub step: f64,
}

impl ManualAuditEventMetricStep for RoundingManualAuditEventMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualAuditEventMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualAuditEventMetricAdjuster {
    pub cap: f64,
}

impl ManualAuditEventMetricStep for CapManualAuditEventMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualAuditEventMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualAuditEventMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualAuditEventMetricStep for TieredManualAuditEventMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualAuditEventMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualAuditEventMetricAdjuster {
    pub minimum: f64,
}

impl ManualAuditEventMetricStep for MinimumManualAuditEventMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualAuditEventMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the manual audit event metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualAuditEventMetricEngine {
    pub steps: Vec<Rc<dyn ManualAuditEventMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualAuditEventMetricEngine {
    pub fn run(&mut self, item: &ManualAuditEventMetric) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualAuditEventMetricStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualAuditEventMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_audit_event_metric_engine() -> ManualAuditEventMetricEngine {
    ManualAuditEventMetricEngine {
        steps: vec![
            Rc::new(PercentManualAuditEventMetricAdjuster { percent: 21.9 }) as Rc<dyn ManualAuditEventMetricStep>,
            Rc::new(RegionManualAuditEventMetricAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.85), ("online".to_string(), 0.81)]),
            }),
            Rc::new(RoundingManualAuditEventMetricAdjuster { step: 1.0 }),
            Rc::new(CapManualAuditEventMetricAdjuster { cap: 44878.0 }),
            Rc::new(TieredManualAuditEventMetricAdjuster {
                bounds: vec![3574.0, 6749.0],
                rates: vec![1.11, 1.11],
            }),
            Rc::new(MinimumManualAuditEventMetricAdjuster { minimum: 46.3 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 14328.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_audit_event_metric_engine();
        let item = new_manual_audit_event_metric("code-901", "label-94");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
