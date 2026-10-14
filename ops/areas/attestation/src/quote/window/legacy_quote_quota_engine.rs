use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy quote quota calculation.
pub trait LegacyQuoteQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyQuoteQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterLegacyQuoteQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyQuoteQuotaAdjuster for ClusterLegacyQuoteQuotaAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &LegacyQuoteQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyQuoteQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyQuoteQuotaAdjuster for FlaggedLegacyQuoteQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyQuoteQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyQuoteQuotaAdjuster {
    pub step: f64,
}

impl LegacyQuoteQuotaAdjuster for RoundingLegacyQuoteQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyQuoteQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyQuoteQuotaAdjuster {
    pub minimum: f64,
}

impl LegacyQuoteQuotaAdjuster for MinimumLegacyQuoteQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyQuoteQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the legacy quote quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyQuoteQuotaEngine {
    pub steps: Vec<Rc<dyn LegacyQuoteQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyQuoteQuotaEngine {
    pub fn evaluate(&mut self, item: &LegacyQuoteQuota) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LegacyQuoteQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_quote_quota_engine(flags: &flags::Registry) -> LegacyQuoteQuotaEngine {
    LegacyQuoteQuotaEngine {
        steps: vec![
            Rc::new(ClusterLegacyQuoteQuotaAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.16),
                    ("south".to_string(), 1.39),
                    ("retail".to_string(), 1.22),
                ]),
            }) as Rc<dyn LegacyQuoteQuotaAdjuster>,
            Rc::new(FlaggedLegacyQuoteQuotaAdjuster {
                flags: flags.clone(),
                flag: "legacy-quote-quota-seasonal".to_string(),
                percent: -4.9,
            }),
            Rc::new(RoundingLegacyQuoteQuotaAdjuster { step: 0.05 }),
            Rc::new(MinimumLegacyQuoteQuotaAdjuster { minimum: 27.8 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 46335.8,
    }
}
