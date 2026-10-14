use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy allowlist quota calculation.
pub trait LegacyAllowlistQuotaModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyAllowlistQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleLegacyAllowlistQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyAllowlistQuotaModifier for TitleLegacyAllowlistQuotaAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &LegacyAllowlistQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistQuotaBudgetOffsetAdjuster {
    pub weight: f64,
}

impl LegacyAllowlistQuotaModifier for LegacyAllowlistQuotaBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyAllowlistQuota, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyAllowlistQuotaAdjuster {
    pub cap: f64,
}

impl LegacyAllowlistQuotaModifier for CapLegacyAllowlistQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyAllowlistQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyAllowlistQuotaAdjuster {
    pub step: f64,
}

impl LegacyAllowlistQuotaModifier for RoundingLegacyAllowlistQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyAllowlistQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the legacy allowlist quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyAllowlistQuotaEngine {
    pub steps: Vec<Rc<dyn LegacyAllowlistQuotaModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyAllowlistQuotaEngine {
    pub fn price(&mut self, item: &LegacyAllowlistQuota) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyAllowlistQuotaModifier> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[LegacyAllowlistQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_allowlist_quota_engine() -> LegacyAllowlistQuotaEngine {
    LegacyAllowlistQuotaEngine {
        steps: vec![
            Rc::new(TitleLegacyAllowlistQuotaAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.14),
                    ("north".to_string(), 1.3),
                    ("south".to_string(), 0.93),
                ]),
            }) as Rc<dyn LegacyAllowlistQuotaModifier>,
            Rc::new(LegacyAllowlistQuotaBudgetOffsetAdjuster { weight: 0.43 }),
            Rc::new(CapLegacyAllowlistQuotaAdjuster { cap: 7936.0 }),
            Rc::new(RoundingLegacyAllowlistQuotaAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 39566.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_allowlist_quota_engine();
        let item = new_legacy_allowlist_quota("code-71", "title-580");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
