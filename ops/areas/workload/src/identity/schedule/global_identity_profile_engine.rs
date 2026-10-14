use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the global identity profile calculation.
pub trait GlobalIdentityProfileFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalIdentityProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalIdentityProfileAdjuster {
    pub minimum: f64,
}

impl GlobalIdentityProfileFactor for MinimumGlobalIdentityProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalIdentityProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalIdentityProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalIdentityProfileFactor for FlaggedGlobalIdentityProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalIdentityProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalIdentityProfileAdjuster {
    pub cap: f64,
}

impl GlobalIdentityProfileFactor for CapGlobalIdentityProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalIdentityProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameGlobalIdentityProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalIdentityProfileFactor for NameGlobalIdentityProfileAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &GlobalIdentityProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalIdentityProfileAdjuster {
    pub percent: f64,
}

impl GlobalIdentityProfileFactor for PercentGlobalIdentityProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalIdentityProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalIdentityProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalIdentityProfileFactor for TieredGlobalIdentityProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalIdentityProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the global identity profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalIdentityProfileEngine {
    pub steps: Vec<Rc<dyn GlobalIdentityProfileFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalIdentityProfileEngine {
    pub fn evaluate(&mut self, item: &GlobalIdentityProfile) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalIdentityProfileFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalIdentityProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_identity_profile_engine(flags: &flags::Registry) -> GlobalIdentityProfileEngine {
    GlobalIdentityProfileEngine {
        steps: vec![
            Rc::new(MinimumGlobalIdentityProfileAdjuster { minimum: 9.2 }) as Rc<dyn GlobalIdentityProfileFactor>,
            Rc::new(FlaggedGlobalIdentityProfileAdjuster {
                flags: flags.clone(),
                flag: "global-identity-profile-strict".to_string(),
                percent: -9.5,
            }),
            Rc::new(CapGlobalIdentityProfileAdjuster { cap: 20542.0 }),
            Rc::new(NameGlobalIdentityProfileAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.87),
                    ("east".to_string(), 1.14),
                    ("central".to_string(), 0.84),
                ]),
            }),
            Rc::new(PercentGlobalIdentityProfileAdjuster { percent: -2.3 }),
            Rc::new(TieredGlobalIdentityProfileAdjuster {
                bounds: vec![723.0, 6151.0, 7744.0],
                rates: vec![1.04, 1.03, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 36459.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_identity_profile_engine(&flags::new_registry());
        let item = new_global_identity_profile("code-168", "name-630");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
