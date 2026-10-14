use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled release agreement calculation.
pub trait ScheduledReleaseAgreementStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledReleaseAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReleaseAgreementWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledReleaseAgreementStep for ScheduledReleaseAgreementWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledReleaseAgreement, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledReleaseAgreementAdjuster {
    pub step: f64,
}

impl ScheduledReleaseAgreementStep for RoundingScheduledReleaseAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledReleaseAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledReleaseAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledReleaseAgreementStep for TieredScheduledReleaseAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledReleaseAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledReleaseAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledReleaseAgreementStep for FlaggedScheduledReleaseAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledReleaseAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitScheduledReleaseAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledReleaseAgreementStep for UnitScheduledReleaseAgreementAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &ScheduledReleaseAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the scheduled release agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledReleaseAgreementEngine {
    pub steps: Vec<Rc<dyn ScheduledReleaseAgreementStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledReleaseAgreementEngine {
    pub fn compute(&mut self, item: &ScheduledReleaseAgreement) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledReleaseAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_release_agreement_engine(flags: &flags::Registry) -> ScheduledReleaseAgreementEngine {
    ScheduledReleaseAgreementEngine {
        steps: vec![
            Rc::new(ScheduledReleaseAgreementWindowDaysOffsetAdjuster { weight: 0.57 })
                as Rc<dyn ScheduledReleaseAgreementStep>,
            Rc::new(RoundingScheduledReleaseAgreementAdjuster { step: 0.1 }),
            Rc::new(TieredScheduledReleaseAgreementAdjuster {
                bounds: vec![856.0, 5304.0, 5676.0, 9118.0],
                rates: vec![0.94, 0.84, 1.08, 1.14],
            }),
            Rc::new(FlaggedScheduledReleaseAgreementAdjuster {
                flags: flags.clone(),
                flag: "scheduled-release-agreement-regional".to_string(),
                percent: -8.4,
            }),
            Rc::new(UnitScheduledReleaseAgreementAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.24),
                    ("central".to_string(), 1.4),
                    ("east".to_string(), 1.29),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 41370.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_release_agreement_engine(&flags::new_registry());
        let item = new_scheduled_release_agreement("code-213", "label-293");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
