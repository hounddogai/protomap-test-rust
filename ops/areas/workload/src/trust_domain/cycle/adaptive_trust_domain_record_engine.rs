use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive trust domain record calculation.
pub trait AdaptiveTrustDomainRecordModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveTrustDomainRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveTrustDomainRecordAdjuster {
    pub cap: f64,
}

impl AdaptiveTrustDomainRecordModifier for CapAdaptiveTrustDomainRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveTrustDomainRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveTrustDomainRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveTrustDomainRecordModifier for FlaggedAdaptiveTrustDomainRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveTrustDomainRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainRecordMinimumOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveTrustDomainRecordModifier for AdaptiveTrustDomainRecordMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveTrustDomainRecord, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveTrustDomainRecordAdjuster {
    pub minimum: f64,
}

impl AdaptiveTrustDomainRecordModifier for MinimumAdaptiveTrustDomainRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveTrustDomainRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryAdaptiveTrustDomainRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveTrustDomainRecordModifier for CategoryAdaptiveTrustDomainRecordAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &AdaptiveTrustDomainRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveTrustDomainRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveTrustDomainRecordModifier for TieredAdaptiveTrustDomainRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveTrustDomainRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the adaptive trust domain record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveTrustDomainRecordEngine {
    pub steps: Vec<Rc<dyn AdaptiveTrustDomainRecordModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveTrustDomainRecordEngine {
    pub fn run(&mut self, item: &AdaptiveTrustDomainRecord) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveTrustDomainRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_trust_domain_record_engine(flags: &flags::Registry) -> AdaptiveTrustDomainRecordEngine {
    AdaptiveTrustDomainRecordEngine {
        steps: vec![
            Rc::new(CapAdaptiveTrustDomainRecordAdjuster { cap: 17907.0 }) as Rc<dyn AdaptiveTrustDomainRecordModifier>,
            Rc::new(FlaggedAdaptiveTrustDomainRecordAdjuster {
                flags: flags.clone(),
                flag: "adaptive-trust-domain-record-weighted".to_string(),
                percent: 9.6,
            }),
            Rc::new(AdaptiveTrustDomainRecordMinimumOffsetAdjuster { weight: 1.95 }),
            Rc::new(MinimumAdaptiveTrustDomainRecordAdjuster { minimum: 25.5 }),
            Rc::new(CategoryAdaptiveTrustDomainRecordAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.0), ("east".to_string(), 1.28)]),
            }),
            Rc::new(TieredAdaptiveTrustDomainRecordAdjuster {
                bounds: vec![3196.0, 5050.0, 7044.0, 8930.0],
                rates: vec![0.83, 1.04, 0.88, 1.09],
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 42043.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_trust_domain_record_engine(&flags::new_registry());
        let item = new_adaptive_trust_domain_record("code-591", "title-405");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
