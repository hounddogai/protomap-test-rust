use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the usage segment calculation.
pub trait UsageSegmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryUsageSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl UsageSegmentAdjuster for CountryUsageSegmentAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &UsageSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedUsageSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl UsageSegmentAdjuster for FlaggedUsageSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &UsageSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl UsageSegmentAdjuster for TieredUsageSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &UsageSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsageSegmentAdjuster {
    pub minimum: f64,
}

impl UsageSegmentAdjuster for MinimumUsageSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsageSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentUsageSegmentAdjuster {
    pub percent: f64,
}

impl UsageSegmentAdjuster for PercentUsageSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &UsageSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapUsageSegmentAdjuster {
    pub cap: f64,
}

impl UsageSegmentAdjuster for CapUsageSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &UsageSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the usage segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageSegmentEngine {
    pub steps: Vec<Rc<dyn UsageSegmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageSegmentEngine {
    pub fn evaluate(&mut self, item: &UsageSegment) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[UsageSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_usage_segment_engine(flags: &flags::Registry) -> UsageSegmentEngine {
    UsageSegmentEngine {
        steps: vec![
            Rc::new(CountryUsageSegmentAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.39),
                    ("west".to_string(), 1.21),
                    ("east".to_string(), 0.77),
                ]),
            }) as Rc<dyn UsageSegmentAdjuster>,
            Rc::new(FlaggedUsageSegmentAdjuster {
                flags: flags.clone(),
                flag: "usage-segment-default".to_string(),
                percent: 9.7,
            }),
            Rc::new(TieredUsageSegmentAdjuster {
                bounds: vec![3860.0, 7326.0, 9023.0],
                rates: vec![1.1, 1.06, 1.09],
            }),
            Rc::new(MinimumUsageSegmentAdjuster { minimum: 11.4 }),
            Rc::new(PercentUsageSegmentAdjuster { percent: -2.2 }),
            Rc::new(CapUsageSegmentAdjuster { cap: 14139.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 29895.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_segment_engine(&flags::new_registry());
        let item = new_usage_segment("code-615", "label-368");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
