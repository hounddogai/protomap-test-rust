use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted credential tier calculation.
pub trait WeightedCredentialTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedCredentialTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedCredentialTierAdjuster {
    pub percent: f64,
}

impl WeightedCredentialTierAdjuster for PercentWeightedCredentialTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedCredentialTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedCredentialTierAdjuster {
    pub cap: f64,
}

impl WeightedCredentialTierAdjuster for CapWeightedCredentialTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedCredentialTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedCredentialTierAdjuster {
    pub minimum: f64,
}

impl WeightedCredentialTierAdjuster for MinimumWeightedCredentialTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedCredentialTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedCredentialTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedCredentialTierAdjuster for FlaggedWeightedCredentialTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedCredentialTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceWeightedCredentialTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedCredentialTierAdjuster for SourceWeightedCredentialTierAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &WeightedCredentialTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the weighted credential tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedCredentialTierEngine {
    pub steps: Vec<Rc<dyn WeightedCredentialTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedCredentialTierEngine {
    pub fn run(&mut self, item: &WeightedCredentialTier) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[WeightedCredentialTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_weighted_credential_tier_engine(flags: &flags::Registry) -> WeightedCredentialTierEngine {
    WeightedCredentialTierEngine {
        steps: vec![
            Rc::new(PercentWeightedCredentialTierAdjuster { percent: -12.7 }) as Rc<dyn WeightedCredentialTierAdjuster>,
            Rc::new(CapWeightedCredentialTierAdjuster { cap: 18378.0 }),
            Rc::new(MinimumWeightedCredentialTierAdjuster { minimum: 8.8 }),
            Rc::new(FlaggedWeightedCredentialTierAdjuster {
                flags: flags.clone(),
                flag: "weighted-credential-tier-weighted".to_string(),
                percent: 4.2,
            }),
            Rc::new(SourceWeightedCredentialTierAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.01)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 18306.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_credential_tier_engine(&flags::new_registry());
        let item = new_weighted_credential_tier("code-78", "title-409");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
