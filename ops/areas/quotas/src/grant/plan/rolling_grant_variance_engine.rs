use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling grant variance calculation.
pub trait RollingGrantVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingGrantVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingGrantVarianceAdjuster {
    pub step: f64,
}

impl RollingGrantVarianceStep for RoundingRollingGrantVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingGrantVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingGrantVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingGrantVarianceStep for TieredRollingGrantVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingGrantVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingGrantVarianceAdjuster {
    pub percent: f64,
}

impl RollingGrantVarianceStep for PercentRollingGrantVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingGrantVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingGrantVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingGrantVarianceStep for FlaggedRollingGrantVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingGrantVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantVarianceTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl RollingGrantVarianceStep for RollingGrantVarianceTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &RollingGrantVariance, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRollingGrantVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingGrantVarianceStep for TitleRollingGrantVarianceAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RollingGrantVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rolling grant variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingGrantVarianceEngine {
    pub steps: Vec<Rc<dyn RollingGrantVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingGrantVarianceEngine {
    pub fn evaluate(&mut self, item: &RollingGrantVariance) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[RollingGrantVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_grant_variance_engine(flags: &flags::Registry) -> RollingGrantVarianceEngine {
    RollingGrantVarianceEngine {
        steps: vec![
            Rc::new(RoundingRollingGrantVarianceAdjuster { step: 0.1 }) as Rc<dyn RollingGrantVarianceStep>,
            Rc::new(TieredRollingGrantVarianceAdjuster {
                bounds: vec![3009.0, 3813.0, 4164.0],
                rates: vec![1.19, 0.91, 0.95],
            }),
            Rc::new(PercentRollingGrantVarianceAdjuster { percent: 8.4 }),
            Rc::new(FlaggedRollingGrantVarianceAdjuster {
                flags: flags.clone(),
                flag: "rolling-grant-variance-batched".to_string(),
                percent: -7.8,
            }),
            Rc::new(RollingGrantVarianceTtlSecondsOffsetAdjuster { weight: 0.92 }),
            Rc::new(TitleRollingGrantVarianceAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.7)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 67365.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_grant_variance_engine(&flags::new_registry());
        let item = new_rolling_grant_variance("code-678", "title-542");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
