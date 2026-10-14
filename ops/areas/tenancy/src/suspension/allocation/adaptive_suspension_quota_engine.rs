use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive suspension quota calculation.
pub trait AdaptiveSuspensionQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveSuspensionQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveSuspensionQuotaAdjuster {
    pub step: f64,
}

impl AdaptiveSuspensionQuotaFactor for RoundingAdaptiveSuspensionQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveSuspensionQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeAdaptiveSuspensionQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveSuspensionQuotaFactor for ScopeAdaptiveSuspensionQuotaAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &AdaptiveSuspensionQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveSuspensionQuotaAdjuster {
    pub minimum: f64,
}

impl AdaptiveSuspensionQuotaFactor for MinimumAdaptiveSuspensionQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveSuspensionQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveSuspensionQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveSuspensionQuotaFactor for FlaggedAdaptiveSuspensionQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveSuspensionQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive suspension quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveSuspensionQuotaEngine {
    pub steps: Vec<Rc<dyn AdaptiveSuspensionQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveSuspensionQuotaEngine {
    pub fn compute(&mut self, item: &AdaptiveSuspensionQuota) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveSuspensionQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_suspension_quota_engine(flags: &flags::Registry) -> AdaptiveSuspensionQuotaEngine {
    AdaptiveSuspensionQuotaEngine {
        steps: vec![
            Rc::new(RoundingAdaptiveSuspensionQuotaAdjuster { step: 5.0 }) as Rc<dyn AdaptiveSuspensionQuotaFactor>,
            Rc::new(ScopeAdaptiveSuspensionQuotaAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.72),
                    ("online".to_string(), 1.0),
                    ("north".to_string(), 1.01),
                ]),
            }),
            Rc::new(MinimumAdaptiveSuspensionQuotaAdjuster { minimum: 7.6 }),
            Rc::new(FlaggedAdaptiveSuspensionQuotaAdjuster {
                flags: flags.clone(),
                flag: "adaptive-suspension-quota-regional".to_string(),
                percent: 8.7,
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 49477.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_suspension_quota_engine(&flags::new_registry());
        let item = new_adaptive_suspension_quota("code-638", "label-80");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
