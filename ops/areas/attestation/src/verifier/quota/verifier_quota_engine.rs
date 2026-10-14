use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the verifier quota calculation.
pub trait VerifierQuotaModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl VerifierQuotaModifier for TieredVerifierQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &VerifierQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedVerifierQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl VerifierQuotaModifier for FlaggedVerifierQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &VerifierQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapVerifierQuotaAdjuster {
    pub cap: f64,
}

impl VerifierQuotaModifier for CapVerifierQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &VerifierQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentVerifierQuotaAdjuster {
    pub percent: f64,
}

impl VerifierQuotaModifier for PercentVerifierQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &VerifierQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierQuotaAdjuster {
    pub step: f64,
}

impl VerifierQuotaModifier for RoundingVerifierQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the verifier quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierQuotaEngine {
    pub steps: Vec<Rc<dyn VerifierQuotaModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierQuotaEngine {
    pub fn run(&mut self, item: &VerifierQuota) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn VerifierQuotaModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[VerifierQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_verifier_quota_engine(flags: &flags::Registry) -> VerifierQuotaEngine {
    VerifierQuotaEngine {
        steps: vec![
            Rc::new(TieredVerifierQuotaAdjuster {
                bounds: vec![7659.0, 9116.0],
                rates: vec![1.16, 1.09],
            }) as Rc<dyn VerifierQuotaModifier>,
            Rc::new(FlaggedVerifierQuotaAdjuster {
                flags: flags.clone(),
                flag: "verifier-quota-scheduled".to_string(),
                percent: 4.5,
            }),
            Rc::new(CapVerifierQuotaAdjuster { cap: 19374.0 }),
            Rc::new(PercentVerifierQuotaAdjuster { percent: 4.5 }),
            Rc::new(RoundingVerifierQuotaAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 5725.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_verifier_quota_engine(&flags::new_registry());
        let item = new_verifier_quota("code-802", "label-920");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
