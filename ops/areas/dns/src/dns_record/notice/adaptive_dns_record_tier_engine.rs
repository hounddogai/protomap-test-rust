use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive dns record tier calculation.
pub trait AdaptiveDnsRecordTierFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveDnsRecordTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDnsRecordTierRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveDnsRecordTierFactor for AdaptiveDnsRecordTierRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveDnsRecordTier, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveDnsRecordTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveDnsRecordTierFactor for TieredAdaptiveDnsRecordTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveDnsRecordTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveDnsRecordTierAdjuster {
    pub minimum: f64,
}

impl AdaptiveDnsRecordTierFactor for MinimumAdaptiveDnsRecordTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveDnsRecordTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveDnsRecordTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveDnsRecordTierFactor for FlaggedAdaptiveDnsRecordTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveDnsRecordTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveDnsRecordTierAdjuster {
    pub step: f64,
}

impl AdaptiveDnsRecordTierFactor for RoundingAdaptiveDnsRecordTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveDnsRecordTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the adaptive dns record tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveDnsRecordTierEngine {
    pub steps: Vec<Rc<dyn AdaptiveDnsRecordTierFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveDnsRecordTierEngine {
    pub fn compute(&mut self, item: &AdaptiveDnsRecordTier) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveDnsRecordTierFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveDnsRecordTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_dns_record_tier_engine(flags: &flags::Registry) -> AdaptiveDnsRecordTierEngine {
    AdaptiveDnsRecordTierEngine {
        steps: vec![
            Rc::new(AdaptiveDnsRecordTierRotationDaysOffsetAdjuster { weight: 0.49 })
                as Rc<dyn AdaptiveDnsRecordTierFactor>,
            Rc::new(TieredAdaptiveDnsRecordTierAdjuster {
                bounds: vec![6460.0, 6583.0],
                rates: vec![0.85, 0.93],
            }),
            Rc::new(MinimumAdaptiveDnsRecordTierAdjuster { minimum: 38.8 }),
            Rc::new(FlaggedAdaptiveDnsRecordTierAdjuster {
                flags: flags.clone(),
                flag: "adaptive-dns-record-tier-batched".to_string(),
                percent: 8.2,
            }),
            Rc::new(RoundingAdaptiveDnsRecordTierAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 99344.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_dns_record_tier_engine(&flags::new_registry());
        let item = new_adaptive_dns_record_tier("code-716", "title-81");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
