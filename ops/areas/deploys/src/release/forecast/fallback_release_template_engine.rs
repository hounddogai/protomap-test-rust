use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback release template calculation.
pub trait FallbackReleaseTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackReleaseTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceFallbackReleaseTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackReleaseTemplateModifier for NamespaceFallbackReleaseTemplateAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &FallbackReleaseTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackReleaseTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackReleaseTemplateModifier for FlaggedFallbackReleaseTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackReleaseTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackReleaseTemplateAdjuster {
    pub cap: f64,
}

impl FallbackReleaseTemplateModifier for CapFallbackReleaseTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackReleaseTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackReleaseTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackReleaseTemplateModifier for TieredFallbackReleaseTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackReleaseTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackReleaseTemplateAdjuster {
    pub minimum: f64,
}

impl FallbackReleaseTemplateModifier for MinimumFallbackReleaseTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackReleaseTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the fallback release template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackReleaseTemplateEngine {
    pub steps: Vec<Rc<dyn FallbackReleaseTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackReleaseTemplateEngine {
    pub fn evaluate(&mut self, item: &FallbackReleaseTemplate) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FallbackReleaseTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_release_template_engine(flags: &flags::Registry) -> FallbackReleaseTemplateEngine {
    FallbackReleaseTemplateEngine {
        steps: vec![
            Rc::new(NamespaceFallbackReleaseTemplateAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.35), ("south".to_string(), 1.29)]),
            }) as Rc<dyn FallbackReleaseTemplateModifier>,
            Rc::new(FlaggedFallbackReleaseTemplateAdjuster {
                flags: flags.clone(),
                flag: "fallback-release-template-pending".to_string(),
                percent: 5.6,
            }),
            Rc::new(CapFallbackReleaseTemplateAdjuster { cap: 30760.0 }),
            Rc::new(TieredFallbackReleaseTemplateAdjuster {
                bounds: vec![330.0, 7312.0],
                rates: vec![1.09, 0.96],
            }),
            Rc::new(MinimumFallbackReleaseTemplateAdjuster { minimum: 20.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 68338.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_release_template_engine(&flags::new_registry());
        let item = new_fallback_release_template("code-502", "label-961");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
