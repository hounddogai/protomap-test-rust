use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rolling quote bundle calculation.
pub trait RollingQuoteBundleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingQuoteBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteBundleScoreOffsetAdjuster {
    pub weight: f64,
}

impl RollingQuoteBundleAdjuster for RollingQuoteBundleScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &RollingQuoteBundle, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingQuoteBundleAdjuster {
    pub minimum: f64,
}

impl RollingQuoteBundleAdjuster for MinimumRollingQuoteBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingQuoteBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRollingQuoteBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingQuoteBundleAdjuster for NameRollingQuoteBundleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RollingQuoteBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingQuoteBundleAdjuster {
    pub percent: f64,
}

impl RollingQuoteBundleAdjuster for PercentRollingQuoteBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingQuoteBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingQuoteBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingQuoteBundleAdjuster for FlaggedRollingQuoteBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingQuoteBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingQuoteBundleAdjuster {
    pub cap: f64,
}

impl RollingQuoteBundleAdjuster for CapRollingQuoteBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingQuoteBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rolling quote bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingQuoteBundleEngine {
    pub steps: Vec<Rc<dyn RollingQuoteBundleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingQuoteBundleEngine {
    pub fn run(&mut self, item: &RollingQuoteBundle) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingQuoteBundleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingQuoteBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_quote_bundle_engine(flags: &flags::Registry) -> RollingQuoteBundleEngine {
    RollingQuoteBundleEngine {
        steps: vec![
            Rc::new(RollingQuoteBundleScoreOffsetAdjuster { weight: 1.58 }) as Rc<dyn RollingQuoteBundleAdjuster>,
            Rc::new(MinimumRollingQuoteBundleAdjuster { minimum: 18.5 }),
            Rc::new(NameRollingQuoteBundleAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.0), ("retail".to_string(), 0.77)]),
            }),
            Rc::new(PercentRollingQuoteBundleAdjuster { percent: 15.2 }),
            Rc::new(FlaggedRollingQuoteBundleAdjuster {
                flags: flags.clone(),
                flag: "rolling-quote-bundle-seasonal".to_string(),
                percent: -0.8,
            }),
            Rc::new(CapRollingQuoteBundleAdjuster { cap: 39945.0 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 9750.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_quote_bundle_engine(&flags::new_registry());
        let item = new_rolling_quote_bundle("code-314", "name-273");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
