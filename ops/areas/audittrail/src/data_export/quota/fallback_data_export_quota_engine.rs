use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback data export quota calculation.
pub trait FallbackDataExportQuotaModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackDataExportQuota, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackDataExportQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackDataExportQuotaModifier for FlaggedFallbackDataExportQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackDataExportQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackDataExportQuotaAdjuster {
    pub cap: f64,
}

impl FallbackDataExportQuotaModifier for CapFallbackDataExportQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackDataExportQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackDataExportQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackDataExportQuotaModifier for TieredFallbackDataExportQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackDataExportQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackDataExportQuotaAdjuster {
    pub step: f64,
}

impl FallbackDataExportQuotaModifier for RoundingFallbackDataExportQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackDataExportQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportQuotaRateOffsetAdjuster {
    pub weight: f64,
}

impl FallbackDataExportQuotaModifier for FallbackDataExportQuotaRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackDataExportQuota, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

/// Runs the fallback data export quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackDataExportQuotaEngine {
    pub steps: Vec<Rc<dyn FallbackDataExportQuotaModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackDataExportQuotaEngine {
    pub fn compute(&mut self, item: &FallbackDataExportQuota) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[FallbackDataExportQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_data_export_quota_engine(flags: &flags::Registry) -> FallbackDataExportQuotaEngine {
    FallbackDataExportQuotaEngine {
        steps: vec![
            Rc::new(FlaggedFallbackDataExportQuotaAdjuster {
                flags: flags.clone(),
                flag: "fallback-data-export-quota-seasonal".to_string(),
                percent: 3.5,
            }) as Rc<dyn FallbackDataExportQuotaModifier>,
            Rc::new(CapFallbackDataExportQuotaAdjuster { cap: 22799.0 }),
            Rc::new(TieredFallbackDataExportQuotaAdjuster {
                bounds: vec![1373.0, 3744.0, 6254.0, 7080.0],
                rates: vec![0.87, 1.18, 0.87, 0.8],
            }),
            Rc::new(RoundingFallbackDataExportQuotaAdjuster { step: 0.1 }),
            Rc::new(FallbackDataExportQuotaRateOffsetAdjuster { weight: 1.92 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 67853.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_data_export_quota_engine(&flags::new_registry());
        let item = new_fallback_data_export_quota("code-993", "label-513");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
