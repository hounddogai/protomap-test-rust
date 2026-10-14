use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary nonce notice calculation.
pub trait PrimaryNonceNoticeFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryNonceNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryNonceNoticeVersionOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryNonceNoticeFactor for PrimaryNonceNoticeVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryNonceNotice, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryNonceNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryNonceNoticeFactor for TieredPrimaryNonceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryNonceNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryNonceNoticeAdjuster {
    pub cap: f64,
}

impl PrimaryNonceNoticeFactor for CapPrimaryNonceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryNonceNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryNonceNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryNonceNoticeFactor for FlaggedPrimaryNonceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryNonceNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryNonceNoticeAdjuster {
    pub minimum: f64,
}

impl PrimaryNonceNoticeFactor for MinimumPrimaryNonceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryNonceNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the primary nonce notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryNonceNoticeEngine {
    pub steps: Vec<Rc<dyn PrimaryNonceNoticeFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryNonceNoticeEngine {
    pub fn compute(&mut self, item: &PrimaryNonceNotice) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryNonceNoticeFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryNonceNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_nonce_notice_engine(flags: &flags::Registry) -> PrimaryNonceNoticeEngine {
    PrimaryNonceNoticeEngine {
        steps: vec![
            Rc::new(PrimaryNonceNoticeVersionOffsetAdjuster { weight: 0.83 }) as Rc<dyn PrimaryNonceNoticeFactor>,
            Rc::new(TieredPrimaryNonceNoticeAdjuster {
                bounds: vec![8039.0, 9584.0],
                rates: vec![0.82, 1.1],
            }),
            Rc::new(CapPrimaryNonceNoticeAdjuster { cap: 46505.0 }),
            Rc::new(FlaggedPrimaryNonceNoticeAdjuster {
                flags: flags.clone(),
                flag: "primary-nonce-notice-default".to_string(),
                percent: 9.6,
            }),
            Rc::new(MinimumPrimaryNonceNoticeAdjuster { minimum: 49.3 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 7706.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_nonce_notice_engine(&flags::new_registry());
        let item = new_primary_nonce_notice("code-66", "label-475");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
