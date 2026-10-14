use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered overage agreement calculation.
pub trait TieredOverageAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredOverageAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredOverageAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredOverageAgreementModifier for FlaggedTieredOverageAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredOverageAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredOverageAgreementAdjuster {
    pub percent: f64,
}

impl TieredOverageAgreementModifier for PercentTieredOverageAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredOverageAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageAgreementBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl TieredOverageAgreementModifier for TieredOverageAgreementBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &TieredOverageAgreement, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredOverageAgreementAdjuster {
    pub cap: f64,
}

impl TieredOverageAgreementModifier for CapTieredOverageAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredOverageAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredOverageAgreementAdjuster {
    pub minimum: f64,
}

impl TieredOverageAgreementModifier for MinimumTieredOverageAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredOverageAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterTieredOverageAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredOverageAgreementModifier for ClusterTieredOverageAgreementAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &TieredOverageAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the tiered overage agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredOverageAgreementEngine {
    pub steps: Vec<Rc<dyn TieredOverageAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredOverageAgreementEngine {
    pub fn price(&mut self, item: &TieredOverageAgreement) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TieredOverageAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_overage_agreement_engine(flags: &flags::Registry) -> TieredOverageAgreementEngine {
    TieredOverageAgreementEngine {
        steps: vec![
            Rc::new(FlaggedTieredOverageAgreementAdjuster {
                flags: flags.clone(),
                flag: "tiered-overage-agreement-regional".to_string(),
                percent: 5.3,
            }) as Rc<dyn TieredOverageAgreementModifier>,
            Rc::new(PercentTieredOverageAgreementAdjuster { percent: 23.6 }),
            Rc::new(TieredOverageAgreementBatchSizeOffsetAdjuster { weight: 0.79 }),
            Rc::new(CapTieredOverageAgreementAdjuster { cap: 44260.0 }),
            Rc::new(MinimumTieredOverageAgreementAdjuster { minimum: 16.1 }),
            Rc::new(ClusterTieredOverageAgreementAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.25), ("north".to_string(), 1.37)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 78363.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_overage_agreement_engine(&flags::new_registry());
        let item = new_tiered_overage_agreement("code-973", "name-412");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
