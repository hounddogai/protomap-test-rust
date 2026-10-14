use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling page metric calculation.
pub trait RollingPageMetricModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingPageMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingPageMetricAdjuster {
    pub step: f64,
}

impl RollingPageMetricModifier for RoundingRollingPageMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingPageMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingPageMetricAdjuster {
    pub percent: f64,
}

impl RollingPageMetricModifier for PercentRollingPageMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingPageMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingPageMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingPageMetricModifier for TieredRollingPageMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingPageMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageMetricBudgetOffsetAdjuster {
    pub weight: f64,
}

impl RollingPageMetricModifier for RollingPageMetricBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &RollingPageMetric, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterRollingPageMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingPageMetricModifier for ClusterRollingPageMetricAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &RollingPageMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingPageMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingPageMetricModifier for FlaggedRollingPageMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingPageMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling page metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingPageMetricEngine {
    pub steps: Vec<Rc<dyn RollingPageMetricModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingPageMetricEngine {
    pub fn compute(&mut self, item: &RollingPageMetric) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RollingPageMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_page_metric_engine(flags: &flags::Registry) -> RollingPageMetricEngine {
    RollingPageMetricEngine {
        steps: vec![
            Rc::new(RoundingRollingPageMetricAdjuster { step: 0.25 }) as Rc<dyn RollingPageMetricModifier>,
            Rc::new(PercentRollingPageMetricAdjuster { percent: -14.2 }),
            Rc::new(TieredRollingPageMetricAdjuster {
                bounds: vec![522.0, 6799.0, 7337.0, 9371.0],
                rates: vec![1.11, 0.83, 1.17, 1.12],
            }),
            Rc::new(RollingPageMetricBudgetOffsetAdjuster { weight: 1.66 }),
            Rc::new(ClusterRollingPageMetricAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.72), ("east".to_string(), 0.91)]),
            }),
            Rc::new(FlaggedRollingPageMetricAdjuster {
                flags: flags.clone(),
                flag: "rolling-page-metric-global".to_string(),
                percent: -9.7,
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 69622.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_page_metric_engine(&flags::new_registry());
        let item = new_rolling_page_metric("code-871", "title-81");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
