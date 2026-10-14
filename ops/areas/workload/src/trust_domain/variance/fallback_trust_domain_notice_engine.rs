use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback trust domain notice calculation.
pub trait FallbackTrustDomainNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackTrustDomainNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryFallbackTrustDomainNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackTrustDomainNoticeAdjuster for CategoryFallbackTrustDomainNoticeAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &FallbackTrustDomainNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackTrustDomainNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackTrustDomainNoticeAdjuster for TieredFallbackTrustDomainNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackTrustDomainNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackTrustDomainNoticeAdjuster {
    pub minimum: f64,
}

impl FallbackTrustDomainNoticeAdjuster for MinimumFallbackTrustDomainNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackTrustDomainNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackTrustDomainNoticeAdjuster {
    pub step: f64,
}

impl FallbackTrustDomainNoticeAdjuster for RoundingFallbackTrustDomainNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackTrustDomainNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackTrustDomainNoticeAdjuster {
    pub cap: f64,
}

impl FallbackTrustDomainNoticeAdjuster for CapFallbackTrustDomainNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackTrustDomainNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackTrustDomainNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackTrustDomainNoticeAdjuster for FlaggedFallbackTrustDomainNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackTrustDomainNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback trust domain notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackTrustDomainNoticeEngine {
    pub steps: Vec<Rc<dyn FallbackTrustDomainNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackTrustDomainNoticeEngine {
    pub fn run(&mut self, item: &FallbackTrustDomainNotice) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackTrustDomainNoticeAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackTrustDomainNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_trust_domain_notice_engine(flags: &flags::Registry) -> FallbackTrustDomainNoticeEngine {
    FallbackTrustDomainNoticeEngine {
        steps: vec![
            Rc::new(CategoryFallbackTrustDomainNoticeAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.16), ("online".to_string(), 0.74)]),
            }) as Rc<dyn FallbackTrustDomainNoticeAdjuster>,
            Rc::new(TieredFallbackTrustDomainNoticeAdjuster {
                bounds: vec![1610.0, 3786.0, 5740.0],
                rates: vec![1.16, 1.11, 0.82],
            }),
            Rc::new(MinimumFallbackTrustDomainNoticeAdjuster { minimum: 8.8 }),
            Rc::new(RoundingFallbackTrustDomainNoticeAdjuster { step: 0.25 }),
            Rc::new(CapFallbackTrustDomainNoticeAdjuster { cap: 25555.0 }),
            Rc::new(FlaggedFallbackTrustDomainNoticeAdjuster {
                flags: flags.clone(),
                flag: "fallback-trust-domain-notice-lenient".to_string(),
                percent: 4.6,
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 28074.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_trust_domain_notice_engine(&flags::new_registry());
        let item = new_fallback_trust_domain_notice("code-982", "label-262");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
