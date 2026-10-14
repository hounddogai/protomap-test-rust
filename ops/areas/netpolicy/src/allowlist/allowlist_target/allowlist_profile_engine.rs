use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the allowlist profile calculation.
pub trait AllowlistProfileFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistProfile, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedAllowlistProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AllowlistProfileFactor for FlaggedAllowlistProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AllowlistProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AllowlistProfileFactor for TieredAllowlistProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AllowlistProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistProfileQuantityOffsetAdjuster {
    pub weight: f64,
}

impl AllowlistProfileFactor for AllowlistProfileQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &AllowlistProfile, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAllowlistProfileAdjuster {
    pub cap: f64,
}

impl AllowlistProfileFactor for CapAllowlistProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AllowlistProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the allowlist profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistProfileEngine {
    pub steps: Vec<Rc<dyn AllowlistProfileFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistProfileEngine {
    pub fn compute(&mut self, item: &AllowlistProfile) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AllowlistProfileFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_allowlist_profile_engine(flags: &flags::Registry) -> AllowlistProfileEngine {
    AllowlistProfileEngine {
        steps: vec![
            Rc::new(FlaggedAllowlistProfileAdjuster {
                flags: flags.clone(),
                flag: "allowlist-profile-global".to_string(),
                percent: 6.7,
            }) as Rc<dyn AllowlistProfileFactor>,
            Rc::new(TieredAllowlistProfileAdjuster {
                bounds: vec![1866.0, 5349.0],
                rates: vec![1.13, 0.89],
            }),
            Rc::new(AllowlistProfileQuantityOffsetAdjuster { weight: 0.88 }),
            Rc::new(CapAllowlistProfileAdjuster { cap: 39582.0 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 96319.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_allowlist_profile_engine(&flags::new_registry());
        let item = new_allowlist_profile("code-894", "title-625");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
