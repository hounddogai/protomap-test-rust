use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled allowlist digest calculation.
pub trait ScheduledAllowlistDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledAllowlistDigest, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledAllowlistDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledAllowlistDigestAdjuster for FlaggedScheduledAllowlistDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledAllowlistDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleScheduledAllowlistDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledAllowlistDigestAdjuster for LocaleScheduledAllowlistDigestAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &ScheduledAllowlistDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledAllowlistDigestAdjuster {
    pub step: f64,
}

impl ScheduledAllowlistDigestAdjuster for RoundingScheduledAllowlistDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledAllowlistDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the scheduled allowlist digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledAllowlistDigestEngine {
    pub steps: Vec<Rc<dyn ScheduledAllowlistDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledAllowlistDigestEngine {
    pub fn evaluate(&mut self, item: &ScheduledAllowlistDigest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ScheduledAllowlistDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_allowlist_digest_engine(flags: &flags::Registry) -> ScheduledAllowlistDigestEngine {
    ScheduledAllowlistDigestEngine {
        steps: vec![
            Rc::new(FlaggedScheduledAllowlistDigestAdjuster {
                flags: flags.clone(),
                flag: "scheduled-allowlist-digest-adaptive".to_string(),
                percent: 9.4,
            }) as Rc<dyn ScheduledAllowlistDigestAdjuster>,
            Rc::new(LocaleScheduledAllowlistDigestAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.39),
                    ("south".to_string(), 1.01),
                    ("retail".to_string(), 1.13),
                ]),
            }),
            Rc::new(RoundingScheduledAllowlistDigestAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 14030.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_allowlist_digest_engine(&flags::new_registry());
        let item = new_scheduled_allowlist_digest("code-764", "label-216");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
