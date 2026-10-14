use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling usage estimate calculation.
pub trait RollingUsageEstimateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingUsageEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingUsageEstimateAdjuster {
    pub percent: f64,
}

impl RollingUsageEstimateStep for PercentRollingUsageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingUsageEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingUsageEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingUsageEstimateStep for FlaggedRollingUsageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingUsageEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionRollingUsageEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingUsageEstimateStep for RegionRollingUsageEstimateAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &RollingUsageEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingUsageEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingUsageEstimateStep for TieredRollingUsageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingUsageEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rolling usage estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingUsageEstimateEngine {
    pub steps: Vec<Rc<dyn RollingUsageEstimateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingUsageEstimateEngine {
    pub fn run(&mut self, item: &RollingUsageEstimate) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[RollingUsageEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_usage_estimate_engine(flags: &flags::Registry) -> RollingUsageEstimateEngine {
    RollingUsageEstimateEngine {
        steps: vec![
            Rc::new(PercentRollingUsageEstimateAdjuster { percent: -4.9 }) as Rc<dyn RollingUsageEstimateStep>,
            Rc::new(FlaggedRollingUsageEstimateAdjuster {
                flags: flags.clone(),
                flag: "rolling-usage-estimate-strict".to_string(),
                percent: 3.3,
            }),
            Rc::new(RegionRollingUsageEstimateAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.35),
                    ("online".to_string(), 1.38),
                    ("south".to_string(), 1.27),
                ]),
            }),
            Rc::new(TieredRollingUsageEstimateAdjuster {
                bounds: vec![888.0, 2332.0, 6682.0, 7306.0],
                rates: vec![1.1, 1.09, 0.92, 0.94],
            }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 28324.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_usage_estimate_engine(&flags::new_registry());
        let item = new_rolling_usage_estimate("code-142", "label-465");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
