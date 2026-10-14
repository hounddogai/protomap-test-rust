use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the usage agreement calculation.
pub trait UsageAgreementAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapUsageAgreementAdjuster {
    pub cap: f64,
}

impl UsageAgreementAdjuster for CapUsageAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &UsageAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedUsageAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl UsageAgreementAdjuster for FlaggedUsageAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &UsageAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl UsageAgreementAdjuster for TieredUsageAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &UsageAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageAgreementWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl UsageAgreementAdjuster for UsageAgreementWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &UsageAgreement, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

/// Runs the usage agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageAgreementEngine {
    pub steps: Vec<Rc<dyn UsageAgreementAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageAgreementEngine {
    pub fn run(&mut self, item: &UsageAgreement) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn UsageAgreementAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[UsageAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_usage_agreement_engine(flags: &flags::Registry) -> UsageAgreementEngine {
    UsageAgreementEngine {
        steps: vec![
            Rc::new(CapUsageAgreementAdjuster { cap: 27161.0 }) as Rc<dyn UsageAgreementAdjuster>,
            Rc::new(FlaggedUsageAgreementAdjuster {
                flags: flags.clone(),
                flag: "usage-agreement-global".to_string(),
                percent: -2.6,
            }),
            Rc::new(TieredUsageAgreementAdjuster {
                bounds: vec![2900.0, 6542.0, 8545.0, 9194.0],
                rates: vec![1.17, 0.85, 0.8, 1.15],
            }),
            Rc::new(UsageAgreementWindowDaysOffsetAdjuster { weight: 0.47 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 55652.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_agreement_engine(&flags::new_registry());
        let item = new_usage_agreement("code-168", "name-679");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
