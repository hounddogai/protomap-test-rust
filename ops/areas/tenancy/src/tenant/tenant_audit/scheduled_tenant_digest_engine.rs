use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled tenant digest calculation.
pub trait ScheduledTenantDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledTenantDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelScheduledTenantDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledTenantDigestAdjuster for LabelScheduledTenantDigestAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ScheduledTenantDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledTenantDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledTenantDigestAdjuster for TieredScheduledTenantDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledTenantDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledTenantDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledTenantDigestAdjuster for FlaggedScheduledTenantDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledTenantDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled tenant digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledTenantDigestEngine {
    pub steps: Vec<Rc<dyn ScheduledTenantDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledTenantDigestEngine {
    pub fn evaluate(&mut self, item: &ScheduledTenantDigest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ScheduledTenantDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_tenant_digest_engine(flags: &flags::Registry) -> ScheduledTenantDigestEngine {
    ScheduledTenantDigestEngine {
        steps: vec![
            Rc::new(LabelScheduledTenantDigestAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.77),
                    ("central".to_string(), 0.77),
                    ("online".to_string(), 0.78),
                ]),
            }) as Rc<dyn ScheduledTenantDigestAdjuster>,
            Rc::new(TieredScheduledTenantDigestAdjuster {
                bounds: vec![3059.0, 5154.0, 9118.0, 9898.0],
                rates: vec![1.17, 0.82, 1.12, 1.09],
            }),
            Rc::new(FlaggedScheduledTenantDigestAdjuster {
                flags: flags.clone(),
                flag: "scheduled-tenant-digest-legacy".to_string(),
                percent: 7.7,
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 2720.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_tenant_digest_engine(&flags::new_registry());
        let item = new_scheduled_tenant_digest("code-922", "label-580");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
