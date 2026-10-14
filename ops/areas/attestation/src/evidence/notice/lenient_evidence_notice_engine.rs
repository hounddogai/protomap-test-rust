use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient evidence notice calculation.
pub trait LenientEvidenceNoticeModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientEvidenceNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientEvidenceNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientEvidenceNoticeModifier for TieredLenientEvidenceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientEvidenceNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientEvidenceNoticeAdjuster {
    pub cap: f64,
}

impl LenientEvidenceNoticeModifier for CapLenientEvidenceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientEvidenceNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientEvidenceNoticeAdjuster {
    pub step: f64,
}

impl LenientEvidenceNoticeModifier for RoundingLenientEvidenceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientEvidenceNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleLenientEvidenceNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientEvidenceNoticeModifier for TitleLenientEvidenceNoticeAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &LenientEvidenceNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientEvidenceNoticeAdjuster {
    pub minimum: f64,
}

impl LenientEvidenceNoticeModifier for MinimumLenientEvidenceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientEvidenceNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientEvidenceNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientEvidenceNoticeModifier for FlaggedLenientEvidenceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientEvidenceNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient evidence notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientEvidenceNoticeEngine {
    pub steps: Vec<Rc<dyn LenientEvidenceNoticeModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientEvidenceNoticeEngine {
    pub fn price(&mut self, item: &LenientEvidenceNotice) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[LenientEvidenceNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_evidence_notice_engine(flags: &flags::Registry) -> LenientEvidenceNoticeEngine {
    LenientEvidenceNoticeEngine {
        steps: vec![
            Rc::new(TieredLenientEvidenceNoticeAdjuster {
                bounds: vec![2880.0, 5790.0],
                rates: vec![1.06, 1.01],
            }) as Rc<dyn LenientEvidenceNoticeModifier>,
            Rc::new(CapLenientEvidenceNoticeAdjuster { cap: 11108.0 }),
            Rc::new(RoundingLenientEvidenceNoticeAdjuster { step: 5.0 }),
            Rc::new(TitleLenientEvidenceNoticeAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.92),
                    ("online".to_string(), 0.95),
                    ("east".to_string(), 0.71),
                ]),
            }),
            Rc::new(MinimumLenientEvidenceNoticeAdjuster { minimum: 32.9 }),
            Rc::new(FlaggedLenientEvidenceNoticeAdjuster {
                flags: flags.clone(),
                flag: "lenient-evidence-notice-tiered".to_string(),
                percent: 2.5,
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 16903.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_evidence_notice_engine(&flags::new_registry());
        let item = new_lenient_evidence_notice("code-345", "title-926");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
