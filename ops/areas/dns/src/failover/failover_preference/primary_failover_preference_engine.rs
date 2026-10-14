use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary failover preference calculation.
pub trait PrimaryFailoverPreferenceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryFailoverPreference, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryFailoverPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryFailoverPreferenceAdjuster for FlaggedPrimaryFailoverPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryFailoverPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePrimaryFailoverPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryFailoverPreferenceAdjuster for SourcePrimaryFailoverPreferenceAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PrimaryFailoverPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryFailoverPreferenceAdjuster {
    pub cap: f64,
}

impl PrimaryFailoverPreferenceAdjuster for CapPrimaryFailoverPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryFailoverPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryFailoverPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryFailoverPreferenceAdjuster for TieredPrimaryFailoverPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryFailoverPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverPreferenceExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryFailoverPreferenceAdjuster for PrimaryFailoverPreferenceExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryFailoverPreference, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

/// Runs the primary failover preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryFailoverPreferenceEngine {
    pub steps: Vec<Rc<dyn PrimaryFailoverPreferenceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryFailoverPreferenceEngine {
    pub fn run(&mut self, item: &PrimaryFailoverPreference) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryFailoverPreferenceAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryFailoverPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_failover_preference_engine(flags: &flags::Registry) -> PrimaryFailoverPreferenceEngine {
    PrimaryFailoverPreferenceEngine {
        steps: vec![
            Rc::new(FlaggedPrimaryFailoverPreferenceAdjuster {
                flags: flags.clone(),
                flag: "primary-failover-preference-primary".to_string(),
                percent: -6.4,
            }) as Rc<dyn PrimaryFailoverPreferenceAdjuster>,
            Rc::new(SourcePrimaryFailoverPreferenceAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.34)]),
            }),
            Rc::new(CapPrimaryFailoverPreferenceAdjuster { cap: 20995.0 }),
            Rc::new(TieredPrimaryFailoverPreferenceAdjuster {
                bounds: vec![867.0, 2462.0, 5048.0, 7852.0],
                rates: vec![1.01, 1.16, 1.0, 0.83],
            }),
            Rc::new(PrimaryFailoverPreferenceExpiresAtOffsetAdjuster { weight: 0.54 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 96655.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_failover_preference_engine(&flags::new_registry());
        let item = new_primary_failover_preference("code-554", "title-363");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
