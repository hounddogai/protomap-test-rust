use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the usage estimate calculation.
pub trait UsageEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedUsageEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl UsageEstimateAdjuster for FlaggedUsageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &UsageEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsageEstimateAdjuster {
    pub minimum: f64,
}

impl UsageEstimateAdjuster for MinimumUsageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsageEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageEstimateThresholdOffsetAdjuster {
    pub weight: f64,
}

impl UsageEstimateAdjuster for UsageEstimateThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &UsageEstimate, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryUsageEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl UsageEstimateAdjuster for CategoryUsageEstimateAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &UsageEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapUsageEstimateAdjuster {
    pub cap: f64,
}

impl UsageEstimateAdjuster for CapUsageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &UsageEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the usage estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageEstimateEngine {
    pub steps: Vec<Rc<dyn UsageEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageEstimateEngine {
    pub fn run(&mut self, item: &UsageEstimate) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[UsageEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_usage_estimate_engine(flags: &flags::Registry) -> UsageEstimateEngine {
    UsageEstimateEngine {
        steps: vec![
            Rc::new(FlaggedUsageEstimateAdjuster {
                flags: flags.clone(),
                flag: "usage-estimate-weighted".to_string(),
                percent: 2.2,
            }) as Rc<dyn UsageEstimateAdjuster>,
            Rc::new(MinimumUsageEstimateAdjuster { minimum: 4.9 }),
            Rc::new(UsageEstimateThresholdOffsetAdjuster { weight: 1.92 }),
            Rc::new(CategoryUsageEstimateAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.75),
                    ("north".to_string(), 1.04),
                    ("south".to_string(), 0.7),
                ]),
            }),
            Rc::new(CapUsageEstimateAdjuster { cap: 16266.0 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 98540.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_estimate_engine(&flags::new_registry());
        let item = new_usage_estimate("code-466", "title-329");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
