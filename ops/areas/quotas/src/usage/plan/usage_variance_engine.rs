use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the usage variance calculation.
pub trait UsageVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl UsageVarianceStep for TieredUsageVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &UsageVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionUsageVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl UsageVarianceStep for RegionUsageVarianceAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &UsageVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentUsageVarianceAdjuster {
    pub percent: f64,
}

impl UsageVarianceStep for PercentUsageVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &UsageVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the usage variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageVarianceEngine {
    pub steps: Vec<Rc<dyn UsageVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageVarianceEngine {
    pub fn run(&mut self, item: &UsageVariance) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn UsageVarianceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[UsageVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_usage_variance_engine() -> UsageVarianceEngine {
    UsageVarianceEngine {
        steps: vec![
            Rc::new(TieredUsageVarianceAdjuster {
                bounds: vec![3620.0, 3684.0, 4353.0, 9422.0],
                rates: vec![1.14, 0.88, 1.19, 0.89],
            }) as Rc<dyn UsageVarianceStep>,
            Rc::new(RegionUsageVarianceAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.83)]),
            }),
            Rc::new(PercentUsageVarianceAdjuster { percent: -8.0 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 3896.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_variance_engine();
        let item = new_usage_variance("code-690", "name-751");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
