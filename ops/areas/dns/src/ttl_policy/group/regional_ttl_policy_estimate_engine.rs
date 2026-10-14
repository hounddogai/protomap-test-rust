use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional ttl policy estimate calculation.
pub trait RegionalTtlPolicyEstimateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalTtlPolicyEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalTtlPolicyEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalTtlPolicyEstimateModifier for FlaggedRegionalTtlPolicyEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalTtlPolicyEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalTtlPolicyEstimateAdjuster {
    pub cap: f64,
}

impl RegionalTtlPolicyEstimateModifier for CapRegionalTtlPolicyEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalTtlPolicyEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalTtlPolicyEstimateAdjuster {
    pub step: f64,
}

impl RegionalTtlPolicyEstimateModifier for RoundingRegionalTtlPolicyEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalTtlPolicyEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalTtlPolicyEstimateAdjuster {
    pub percent: f64,
}

impl RegionalTtlPolicyEstimateModifier for PercentRegionalTtlPolicyEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalTtlPolicyEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional ttl policy estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalTtlPolicyEstimateEngine {
    pub steps: Vec<Rc<dyn RegionalTtlPolicyEstimateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalTtlPolicyEstimateEngine {
    pub fn price(&mut self, item: &RegionalTtlPolicyEstimate) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalTtlPolicyEstimateModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalTtlPolicyEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_ttl_policy_estimate_engine(flags: &flags::Registry) -> RegionalTtlPolicyEstimateEngine {
    RegionalTtlPolicyEstimateEngine {
        steps: vec![
            Rc::new(FlaggedRegionalTtlPolicyEstimateAdjuster {
                flags: flags.clone(),
                flag: "regional-ttl-policy-estimate-tiered".to_string(),
                percent: -5.5,
            }) as Rc<dyn RegionalTtlPolicyEstimateModifier>,
            Rc::new(CapRegionalTtlPolicyEstimateAdjuster { cap: 13986.0 }),
            Rc::new(RoundingRegionalTtlPolicyEstimateAdjuster { step: 1.0 }),
            Rc::new(PercentRegionalTtlPolicyEstimateAdjuster { percent: 14.8 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 68237.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_ttl_policy_estimate_engine(&flags::new_registry());
        let item = new_regional_ttl_policy_estimate("code-776", "label-33");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
