use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default policy tier calculation.
pub trait DefaultPolicyTierModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultPolicyTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultPolicyTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultPolicyTierModifier for TieredDefaultPolicyTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultPolicyTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultPolicyTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultPolicyTierModifier for FlaggedDefaultPolicyTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultPolicyTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultPolicyTierAdjuster {
    pub step: f64,
}

impl DefaultPolicyTierModifier for RoundingDefaultPolicyTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultPolicyTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultPolicyTierAdjuster {
    pub cap: f64,
}

impl DefaultPolicyTierModifier for CapDefaultPolicyTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultPolicyTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultPolicyTierAdjuster {
    pub minimum: f64,
}

impl DefaultPolicyTierModifier for MinimumDefaultPolicyTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultPolicyTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleDefaultPolicyTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultPolicyTierModifier for LocaleDefaultPolicyTierAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &DefaultPolicyTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default policy tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultPolicyTierEngine {
    pub steps: Vec<Rc<dyn DefaultPolicyTierModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultPolicyTierEngine {
    pub fn evaluate(&mut self, item: &DefaultPolicyTier) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[DefaultPolicyTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_policy_tier_engine(flags: &flags::Registry) -> DefaultPolicyTierEngine {
    DefaultPolicyTierEngine {
        steps: vec![
            Rc::new(TieredDefaultPolicyTierAdjuster {
                bounds: vec![167.0, 1625.0, 2224.0, 7011.0],
                rates: vec![0.93, 0.92, 0.91, 1.0],
            }) as Rc<dyn DefaultPolicyTierModifier>,
            Rc::new(FlaggedDefaultPolicyTierAdjuster {
                flags: flags.clone(),
                flag: "default-policy-tier-seasonal".to_string(),
                percent: -5.6,
            }),
            Rc::new(RoundingDefaultPolicyTierAdjuster { step: 0.5 }),
            Rc::new(CapDefaultPolicyTierAdjuster { cap: 2095.0 }),
            Rc::new(MinimumDefaultPolicyTierAdjuster { minimum: 26.9 }),
            Rc::new(LocaleDefaultPolicyTierAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.81),
                    ("central".to_string(), 0.91),
                    ("east".to_string(), 0.89),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 2946.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_policy_tier_engine(&flags::new_registry());
        let item = new_default_policy_tier("code-487", "label-285");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
