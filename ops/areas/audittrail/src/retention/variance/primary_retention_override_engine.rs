use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary retention override calculation.
pub trait PrimaryRetentionOverrideStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryRetentionOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryRetentionOverrideAdjuster {
    pub step: f64,
}

impl PrimaryRetentionOverrideStep for RoundingPrimaryRetentionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryRetentionOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryRetentionOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryRetentionOverrideStep for FlaggedPrimaryRetentionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryRetentionOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryRetentionOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryRetentionOverrideStep for TieredPrimaryRetentionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryRetentionOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionOverrideExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryRetentionOverrideStep for PrimaryRetentionOverrideExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryRetentionOverride, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePrimaryRetentionOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryRetentionOverrideStep for TitlePrimaryRetentionOverrideAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PrimaryRetentionOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary retention override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryRetentionOverrideEngine {
    pub steps: Vec<Rc<dyn PrimaryRetentionOverrideStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryRetentionOverrideEngine {
    pub fn evaluate(&mut self, item: &PrimaryRetentionOverride) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryRetentionOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_retention_override_engine(flags: &flags::Registry) -> PrimaryRetentionOverrideEngine {
    PrimaryRetentionOverrideEngine {
        steps: vec![
            Rc::new(RoundingPrimaryRetentionOverrideAdjuster { step: 0.5 }) as Rc<dyn PrimaryRetentionOverrideStep>,
            Rc::new(FlaggedPrimaryRetentionOverrideAdjuster {
                flags: flags.clone(),
                flag: "primary-retention-override-pending".to_string(),
                percent: -7.8,
            }),
            Rc::new(TieredPrimaryRetentionOverrideAdjuster {
                bounds: vec![4580.0, 7512.0, 8412.0],
                rates: vec![0.82, 0.8, 0.94],
            }),
            Rc::new(PrimaryRetentionOverrideExpiresAtOffsetAdjuster { weight: 0.74 }),
            Rc::new(TitlePrimaryRetentionOverrideAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.14), ("central".to_string(), 1.32)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 16705.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_retention_override_engine(&flags::new_registry());
        let item = new_primary_retention_override("code-420", "title-542");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
