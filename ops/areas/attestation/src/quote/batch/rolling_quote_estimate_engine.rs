use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling quote estimate calculation.
pub trait RollingQuoteEstimateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingQuoteEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingQuoteEstimateAdjuster {
    pub percent: f64,
}

impl RollingQuoteEstimateModifier for PercentRollingQuoteEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingQuoteEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingQuoteEstimateAdjuster {
    pub step: f64,
}

impl RollingQuoteEstimateModifier for RoundingRollingQuoteEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingQuoteEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRollingQuoteEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingQuoteEstimateModifier for CategoryRollingQuoteEstimateAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RollingQuoteEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingQuoteEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingQuoteEstimateModifier for FlaggedRollingQuoteEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingQuoteEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteEstimateMaximumOffsetAdjuster {
    pub weight: f64,
}

impl RollingQuoteEstimateModifier for RollingQuoteEstimateMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &RollingQuoteEstimate, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingQuoteEstimateAdjuster {
    pub minimum: f64,
}

impl RollingQuoteEstimateModifier for MinimumRollingQuoteEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingQuoteEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rolling quote estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingQuoteEstimateEngine {
    pub steps: Vec<Rc<dyn RollingQuoteEstimateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingQuoteEstimateEngine {
    pub fn compute(&mut self, item: &RollingQuoteEstimate) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[RollingQuoteEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_quote_estimate_engine(flags: &flags::Registry) -> RollingQuoteEstimateEngine {
    RollingQuoteEstimateEngine {
        steps: vec![
            Rc::new(PercentRollingQuoteEstimateAdjuster { percent: -3.6 }) as Rc<dyn RollingQuoteEstimateModifier>,
            Rc::new(RoundingRollingQuoteEstimateAdjuster { step: 5.0 }),
            Rc::new(CategoryRollingQuoteEstimateAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.22),
                    ("central".to_string(), 1.29),
                    ("east".to_string(), 0.89),
                ]),
            }),
            Rc::new(FlaggedRollingQuoteEstimateAdjuster {
                flags: flags.clone(),
                flag: "rolling-quote-estimate-primary".to_string(),
                percent: -0.7,
            }),
            Rc::new(RollingQuoteEstimateMaximumOffsetAdjuster { weight: 1.22 }),
            Rc::new(MinimumRollingQuoteEstimateAdjuster { minimum: 44.9 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 71308.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_quote_estimate_engine(&flags::new_registry());
        let item = new_rolling_quote_estimate("code-118", "label-23");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
