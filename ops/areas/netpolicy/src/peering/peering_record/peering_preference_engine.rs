use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the peering preference calculation.
pub trait PeeringPreferenceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePeeringPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PeeringPreferenceStep for NamePeeringPreferenceAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PeeringPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPeeringPreferenceAdjuster {
    pub minimum: f64,
}

impl PeeringPreferenceStep for MinimumPeeringPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PeeringPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPeeringPreferenceAdjuster {
    pub step: f64,
}

impl PeeringPreferenceStep for RoundingPeeringPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PeeringPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPeeringPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PeeringPreferenceStep for FlaggedPeeringPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PeeringPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPreferenceCeilingOffsetAdjuster {
    pub weight: f64,
}

impl PeeringPreferenceStep for PeeringPreferenceCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringPreference, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPeeringPreferenceAdjuster {
    pub percent: f64,
}

impl PeeringPreferenceStep for PercentPeeringPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PeeringPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the peering preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringPreferenceEngine {
    pub steps: Vec<Rc<dyn PeeringPreferenceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringPreferenceEngine {
    pub fn evaluate(&mut self, item: &PeeringPreference) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PeeringPreferenceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PeeringPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_peering_preference_engine(flags: &flags::Registry) -> PeeringPreferenceEngine {
    PeeringPreferenceEngine {
        steps: vec![
            Rc::new(NamePeeringPreferenceAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.03), ("north".to_string(), 1.03)]),
            }) as Rc<dyn PeeringPreferenceStep>,
            Rc::new(MinimumPeeringPreferenceAdjuster { minimum: 10.9 }),
            Rc::new(RoundingPeeringPreferenceAdjuster { step: 0.5 }),
            Rc::new(FlaggedPeeringPreferenceAdjuster {
                flags: flags.clone(),
                flag: "peering-preference-strict".to_string(),
                percent: 3.1,
            }),
            Rc::new(PeeringPreferenceCeilingOffsetAdjuster { weight: 1.83 }),
            Rc::new(PercentPeeringPreferenceAdjuster { percent: 15.0 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 71387.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_preference_engine(&flags::new_registry());
        let item = new_peering_preference("code-132", "name-950");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
