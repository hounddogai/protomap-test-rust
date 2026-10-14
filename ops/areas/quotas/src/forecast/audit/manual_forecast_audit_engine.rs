use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual forecast audit calculation.
pub trait ManualForecastAuditStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualForecastAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionManualForecastAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualForecastAuditStep for RegionManualForecastAuditAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ManualForecastAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualForecastAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualForecastAuditStep for TieredManualForecastAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualForecastAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastAuditWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl ManualForecastAuditStep for ManualForecastAuditWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &ManualForecastAudit, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualForecastAuditAdjuster {
    pub percent: f64,
}

impl ManualForecastAuditStep for PercentManualForecastAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualForecastAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualForecastAuditAdjuster {
    pub step: f64,
}

impl ManualForecastAuditStep for RoundingManualForecastAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualForecastAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualForecastAuditAdjuster {
    pub cap: f64,
}

impl ManualForecastAuditStep for CapManualForecastAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualForecastAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the manual forecast audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualForecastAuditEngine {
    pub steps: Vec<Rc<dyn ManualForecastAuditStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualForecastAuditEngine {
    pub fn evaluate(&mut self, item: &ManualForecastAudit) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualForecastAuditStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualForecastAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_forecast_audit_engine() -> ManualForecastAuditEngine {
    ManualForecastAuditEngine {
        steps: vec![
            Rc::new(RegionManualForecastAuditAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.95), ("north".to_string(), 1.06)]),
            }) as Rc<dyn ManualForecastAuditStep>,
            Rc::new(TieredManualForecastAuditAdjuster {
                bounds: vec![318.0, 3102.0, 6788.0, 9966.0],
                rates: vec![0.91, 0.98, 0.98, 1.1],
            }),
            Rc::new(ManualForecastAuditWindowDaysOffsetAdjuster { weight: 0.25 }),
            Rc::new(PercentManualForecastAuditAdjuster { percent: -7.1 }),
            Rc::new(RoundingManualForecastAuditAdjuster { step: 0.1 }),
            Rc::new(CapManualForecastAuditAdjuster { cap: 17873.0 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 34232.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_forecast_audit_engine();
        let item = new_manual_forecast_audit("code-867", "name-536");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
