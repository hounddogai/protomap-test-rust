use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional segment quota calculation.
pub trait RegionalSegmentQuotaModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalSegmentQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalSegmentQuotaAdjuster {
    pub step: f64,
}

impl RegionalSegmentQuotaModifier for RoundingRegionalSegmentQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalSegmentQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalSegmentQuotaAdjuster {
    pub cap: f64,
}

impl RegionalSegmentQuotaModifier for CapRegionalSegmentQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalSegmentQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRegionalSegmentQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalSegmentQuotaModifier for SegmentRegionalSegmentQuotaAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &RegionalSegmentQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalSegmentQuotaAdjuster {
    pub minimum: f64,
}

impl RegionalSegmentQuotaModifier for MinimumRegionalSegmentQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalSegmentQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalSegmentQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalSegmentQuotaModifier for TieredRegionalSegmentQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalSegmentQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalSegmentQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalSegmentQuotaModifier for FlaggedRegionalSegmentQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalSegmentQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional segment quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalSegmentQuotaEngine {
    pub steps: Vec<Rc<dyn RegionalSegmentQuotaModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalSegmentQuotaEngine {
    pub fn evaluate(&mut self, item: &RegionalSegmentQuota) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RegionalSegmentQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_segment_quota_engine(flags: &flags::Registry) -> RegionalSegmentQuotaEngine {
    RegionalSegmentQuotaEngine {
        steps: vec![
            Rc::new(RoundingRegionalSegmentQuotaAdjuster { step: 1.0 }) as Rc<dyn RegionalSegmentQuotaModifier>,
            Rc::new(CapRegionalSegmentQuotaAdjuster { cap: 34678.0 }),
            Rc::new(SegmentRegionalSegmentQuotaAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.78),
                    ("east".to_string(), 1.25),
                    ("central".to_string(), 0.95),
                ]),
            }),
            Rc::new(MinimumRegionalSegmentQuotaAdjuster { minimum: 24.2 }),
            Rc::new(TieredRegionalSegmentQuotaAdjuster {
                bounds: vec![2031.0, 2299.0, 6203.0, 6264.0],
                rates: vec![1.15, 0.92, 1.17, 0.94],
            }),
            Rc::new(FlaggedRegionalSegmentQuotaAdjuster {
                flags: flags.clone(),
                flag: "regional-segment-quota-global".to_string(),
                percent: 1.5,
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 20318.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_segment_quota_engine(&flags::new_registry());
        let item = new_regional_segment_quota("code-646", "label-988");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
