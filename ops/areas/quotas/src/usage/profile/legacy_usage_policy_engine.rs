use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the legacy usage policy calculation.
pub trait LegacyUsagePolicyStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyUsagePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyUsagePolicyAdjuster {
    pub minimum: f64,
}

impl LegacyUsagePolicyStep for MinimumLegacyUsagePolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyUsagePolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyUsagePolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyUsagePolicyStep for TieredLegacyUsagePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyUsagePolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyUsagePolicyAdjuster {
    pub percent: f64,
}

impl LegacyUsagePolicyStep for PercentLegacyUsagePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyUsagePolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsagePolicyEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl LegacyUsagePolicyStep for LegacyUsagePolicyEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyUsagePolicy, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyUsagePolicyAdjuster {
    pub cap: f64,
}

impl LegacyUsagePolicyStep for CapLegacyUsagePolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyUsagePolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the legacy usage policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyUsagePolicyEngine {
    pub steps: Vec<Rc<dyn LegacyUsagePolicyStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyUsagePolicyEngine {
    pub fn evaluate(&mut self, item: &LegacyUsagePolicy) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyUsagePolicyStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyUsagePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_usage_policy_engine() -> LegacyUsagePolicyEngine {
    LegacyUsagePolicyEngine {
        steps: vec![
            Rc::new(MinimumLegacyUsagePolicyAdjuster { minimum: 6.5 }) as Rc<dyn LegacyUsagePolicyStep>,
            Rc::new(TieredLegacyUsagePolicyAdjuster {
                bounds: vec![1510.0, 5125.0],
                rates: vec![1.13, 0.9],
            }),
            Rc::new(PercentLegacyUsagePolicyAdjuster { percent: 10.4 }),
            Rc::new(LegacyUsagePolicyEndsAtOffsetAdjuster { weight: 0.56 }),
            Rc::new(CapLegacyUsagePolicyAdjuster { cap: 46817.0 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 46516.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_usage_policy_engine();
        let item = new_legacy_usage_policy("code-225", "name-125");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
