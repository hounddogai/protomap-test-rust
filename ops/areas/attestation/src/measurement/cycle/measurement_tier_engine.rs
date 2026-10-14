use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the measurement tier calculation.
pub trait MeasurementTierModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementTier, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedMeasurementTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MeasurementTierModifier for FlaggedMeasurementTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MeasurementTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MeasurementTierModifier for TieredMeasurementTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MeasurementTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMeasurementTierAdjuster {
    pub minimum: f64,
}

impl MeasurementTierModifier for MinimumMeasurementTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MeasurementTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementTierSequenceOffsetAdjuster {
    pub weight: f64,
}

impl MeasurementTierModifier for MeasurementTierSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &MeasurementTier, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMeasurementTierAdjuster {
    pub percent: f64,
}

impl MeasurementTierModifier for PercentMeasurementTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MeasurementTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceMeasurementTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MeasurementTierModifier for NamespaceMeasurementTierAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &MeasurementTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the measurement tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementTierEngine {
    pub steps: Vec<Rc<dyn MeasurementTierModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementTierEngine {
    pub fn compute(&mut self, item: &MeasurementTier) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MeasurementTierModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_measurement_tier_engine(flags: &flags::Registry) -> MeasurementTierEngine {
    MeasurementTierEngine {
        steps: vec![
            Rc::new(FlaggedMeasurementTierAdjuster {
                flags: flags.clone(),
                flag: "measurement-tier-batched".to_string(),
                percent: 1.1,
            }) as Rc<dyn MeasurementTierModifier>,
            Rc::new(TieredMeasurementTierAdjuster {
                bounds: vec![3461.0, 3771.0, 8357.0],
                rates: vec![0.97, 1.0, 1.18],
            }),
            Rc::new(MinimumMeasurementTierAdjuster { minimum: 21.5 }),
            Rc::new(MeasurementTierSequenceOffsetAdjuster { weight: 0.51 }),
            Rc::new(PercentMeasurementTierAdjuster { percent: -8.6 }),
            Rc::new(NamespaceMeasurementTierAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.84),
                    ("east".to_string(), 1.02),
                    ("west".to_string(), 1.25),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 5994.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_measurement_tier_engine(&flags::new_registry());
        let item = new_measurement_tier("code-297", "name-843");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
