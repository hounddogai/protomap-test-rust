use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the postmortem quota calculation.
pub trait PostmortemQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPostmortemQuotaAdjuster {
    pub percent: f64,
}

impl PostmortemQuotaFactor for PercentPostmortemQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PostmortemQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopePostmortemQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PostmortemQuotaFactor for ScopePostmortemQuotaAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &PostmortemQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPostmortemQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PostmortemQuotaFactor for FlaggedPostmortemQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PostmortemQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPostmortemQuotaAdjuster {
    pub cap: f64,
}

impl PostmortemQuotaFactor for CapPostmortemQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PostmortemQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the postmortem quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemQuotaEngine {
    pub steps: Vec<Rc<dyn PostmortemQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemQuotaEngine {
    pub fn compute(&mut self, item: &PostmortemQuota) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_postmortem_quota_engine(flags: &flags::Registry) -> PostmortemQuotaEngine {
    PostmortemQuotaEngine {
        steps: vec![
            Rc::new(PercentPostmortemQuotaAdjuster { percent: 1.2 }) as Rc<dyn PostmortemQuotaFactor>,
            Rc::new(ScopePostmortemQuotaAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.13), ("south".to_string(), 1.39)]),
            }),
            Rc::new(FlaggedPostmortemQuotaAdjuster {
                flags: flags.clone(),
                flag: "postmortem-quota-regional".to_string(),
                percent: 2.8,
            }),
            Rc::new(CapPostmortemQuotaAdjuster { cap: 15144.0 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 20257.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_postmortem_quota_engine(&flags::new_registry());
        let item = new_postmortem_quota("code-65", "label-918");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
