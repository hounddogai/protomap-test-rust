use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the freeze draft calculation.
pub trait FreezeDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeDraft, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFreezeDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FreezeDraftModifier for FlaggedFreezeDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FreezeDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeDraftAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl FreezeDraftModifier for FreezeDraftAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeDraft, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FreezeDraftModifier for TieredFreezeDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FreezeDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFreezeDraftAdjuster {
    pub percent: f64,
}

impl FreezeDraftModifier for PercentFreezeDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FreezeDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryFreezeDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeDraftModifier for CountryFreezeDraftAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &FreezeDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFreezeDraftAdjuster {
    pub step: f64,
}

impl FreezeDraftModifier for RoundingFreezeDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FreezeDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the freeze draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeDraftEngine {
    pub steps: Vec<Rc<dyn FreezeDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeDraftEngine {
    pub fn compute(&mut self, item: &FreezeDraft) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FreezeDraftModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FreezeDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_freeze_draft_engine(flags: &flags::Registry) -> FreezeDraftEngine {
    FreezeDraftEngine {
        steps: vec![
            Rc::new(FlaggedFreezeDraftAdjuster {
                flags: flags.clone(),
                flag: "freeze-draft-weighted".to_string(),
                percent: 0.5,
            }) as Rc<dyn FreezeDraftModifier>,
            Rc::new(FreezeDraftAttemptsOffsetAdjuster { weight: 1.22 }),
            Rc::new(TieredFreezeDraftAdjuster {
                bounds: vec![1012.0, 2631.0, 4505.0, 6091.0],
                rates: vec![1.18, 0.97, 0.92, 1.19],
            }),
            Rc::new(PercentFreezeDraftAdjuster { percent: -10.3 }),
            Rc::new(CountryFreezeDraftAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.14),
                    ("west".to_string(), 1.07),
                    ("north".to_string(), 1.2),
                ]),
            }),
            Rc::new(RoundingFreezeDraftAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 67506.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_draft_engine(&flags::new_registry());
        let item = new_freeze_draft("code-665", "name-801");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
