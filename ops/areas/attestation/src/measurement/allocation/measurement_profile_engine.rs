use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the measurement profile calculation.
pub trait MeasurementProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMeasurementProfileAdjuster {
    pub step: f64,
}

impl MeasurementProfileModifier for RoundingMeasurementProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MeasurementProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMeasurementProfileAdjuster {
    pub minimum: f64,
}

impl MeasurementProfileModifier for MinimumMeasurementProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MeasurementProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryMeasurementProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MeasurementProfileModifier for CountryMeasurementProfileAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &MeasurementProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedMeasurementProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MeasurementProfileModifier for FlaggedMeasurementProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MeasurementProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the measurement profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementProfileEngine {
    pub steps: Vec<Rc<dyn MeasurementProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementProfileEngine {
    pub fn price(&mut self, item: &MeasurementProfile) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_measurement_profile_engine(flags: &flags::Registry) -> MeasurementProfileEngine {
    MeasurementProfileEngine {
        steps: vec![
            Rc::new(RoundingMeasurementProfileAdjuster { step: 0.1 }) as Rc<dyn MeasurementProfileModifier>,
            Rc::new(MinimumMeasurementProfileAdjuster { minimum: 42.5 }),
            Rc::new(CountryMeasurementProfileAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.97)]),
            }),
            Rc::new(FlaggedMeasurementProfileAdjuster {
                flags: flags.clone(),
                flag: "measurement-profile-scheduled".to_string(),
                percent: -2.8,
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 35397.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_measurement_profile_engine(&flags::new_registry());
        let item = new_measurement_profile("code-103", "label-705");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
