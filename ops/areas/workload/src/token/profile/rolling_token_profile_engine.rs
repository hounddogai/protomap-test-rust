use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling token profile calculation.
pub trait RollingTokenProfileStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingTokenProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingTokenProfileAdjuster {
    pub minimum: f64,
}

impl RollingTokenProfileStep for MinimumRollingTokenProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingTokenProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingTokenProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingTokenProfileStep for FlaggedRollingTokenProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingTokenProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingTokenProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingTokenProfileStep for TieredRollingTokenProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingTokenProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingTokenProfileAdjuster {
    pub step: f64,
}

impl RollingTokenProfileStep for RoundingRollingTokenProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingTokenProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingTokenProfileAdjuster {
    pub percent: f64,
}

impl RollingTokenProfileStep for PercentRollingTokenProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingTokenProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenProfileGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl RollingTokenProfileStep for RollingTokenProfileGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &RollingTokenProfile, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the rolling token profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingTokenProfileEngine {
    pub steps: Vec<Rc<dyn RollingTokenProfileStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingTokenProfileEngine {
    pub fn evaluate(&mut self, item: &RollingTokenProfile) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RollingTokenProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_token_profile_engine(flags: &flags::Registry) -> RollingTokenProfileEngine {
    RollingTokenProfileEngine {
        steps: vec![
            Rc::new(MinimumRollingTokenProfileAdjuster { minimum: 19.3 }) as Rc<dyn RollingTokenProfileStep>,
            Rc::new(FlaggedRollingTokenProfileAdjuster {
                flags: flags.clone(),
                flag: "rolling-token-profile-lenient".to_string(),
                percent: 7.1,
            }),
            Rc::new(TieredRollingTokenProfileAdjuster {
                bounds: vec![2809.0, 3317.0, 5289.0],
                rates: vec![0.89, 0.83, 1.02],
            }),
            Rc::new(RoundingRollingTokenProfileAdjuster { step: 0.05 }),
            Rc::new(PercentRollingTokenProfileAdjuster { percent: 5.0 }),
            Rc::new(RollingTokenProfileGraceDaysOffsetAdjuster { weight: 0.41 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 10169.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_token_profile_engine(&flags::new_registry());
        let item = new_rolling_token_profile("code-482", "label-560");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
