use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional silence variance calculation.
pub trait RegionalSilenceVarianceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalSilenceVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalSilenceVarianceAdjuster {
    pub step: f64,
}

impl RegionalSilenceVarianceFactor for RoundingRegionalSilenceVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalSilenceVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalSilenceVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalSilenceVarianceFactor for FlaggedRegionalSilenceVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalSilenceVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalSilenceVarianceAdjuster {
    pub cap: f64,
}

impl RegionalSilenceVarianceFactor for CapRegionalSilenceVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalSilenceVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceRegionalSilenceVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalSilenceVarianceFactor for NamespaceRegionalSilenceVarianceAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &RegionalSilenceVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the regional silence variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalSilenceVarianceEngine {
    pub steps: Vec<Rc<dyn RegionalSilenceVarianceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalSilenceVarianceEngine {
    pub fn price(&mut self, item: &RegionalSilenceVariance) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[RegionalSilenceVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_silence_variance_engine(flags: &flags::Registry) -> RegionalSilenceVarianceEngine {
    RegionalSilenceVarianceEngine {
        steps: vec![
            Rc::new(RoundingRegionalSilenceVarianceAdjuster { step: 0.05 }) as Rc<dyn RegionalSilenceVarianceFactor>,
            Rc::new(FlaggedRegionalSilenceVarianceAdjuster {
                flags: flags.clone(),
                flag: "regional-silence-variance-pending".to_string(),
                percent: -0.1,
            }),
            Rc::new(CapRegionalSilenceVarianceAdjuster { cap: 5943.0 }),
            Rc::new(NamespaceRegionalSilenceVarianceAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.07), ("central".to_string(), 1.25)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 50754.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_silence_variance_engine(&flags::new_registry());
        let item = new_regional_silence_variance("code-724", "label-363");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
