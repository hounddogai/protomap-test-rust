use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default delegation override calculation.
pub trait DefaultDelegationOverrideStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultDelegationOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameDefaultDelegationOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultDelegationOverrideStep for NameDefaultDelegationOverrideAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &DefaultDelegationOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultDelegationOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultDelegationOverrideStep for TieredDefaultDelegationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultDelegationOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultDelegationOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultDelegationOverrideStep for FlaggedDefaultDelegationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultDelegationOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultDelegationOverrideAdjuster {
    pub minimum: f64,
}

impl DefaultDelegationOverrideStep for MinimumDefaultDelegationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultDelegationOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the default delegation override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultDelegationOverrideEngine {
    pub steps: Vec<Rc<dyn DefaultDelegationOverrideStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultDelegationOverrideEngine {
    pub fn price(&mut self, item: &DefaultDelegationOverride) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[DefaultDelegationOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_delegation_override_engine(flags: &flags::Registry) -> DefaultDelegationOverrideEngine {
    DefaultDelegationOverrideEngine {
        steps: vec![
            Rc::new(NameDefaultDelegationOverrideAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.9), ("west".to_string(), 0.76)]),
            }) as Rc<dyn DefaultDelegationOverrideStep>,
            Rc::new(TieredDefaultDelegationOverrideAdjuster {
                bounds: vec![2933.0, 7248.0, 8317.0],
                rates: vec![1.19, 0.83, 0.99],
            }),
            Rc::new(FlaggedDefaultDelegationOverrideAdjuster {
                flags: flags.clone(),
                flag: "default-delegation-override-scheduled".to_string(),
                percent: 5.7,
            }),
            Rc::new(MinimumDefaultDelegationOverrideAdjuster { minimum: 25.0 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 79680.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_delegation_override_engine(&flags::new_registry());
        let item = new_default_delegation_override("code-715", "name-15");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
