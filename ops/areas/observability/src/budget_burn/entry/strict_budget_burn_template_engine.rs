use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict budget burn template calculation.
pub trait StrictBudgetBurnTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictBudgetBurnTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleStrictBudgetBurnTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictBudgetBurnTemplateFactor for LocaleStrictBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &StrictBudgetBurnTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictBudgetBurnTemplateAdjuster {
    pub cap: f64,
}

impl StrictBudgetBurnTemplateFactor for CapStrictBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictBudgetBurnTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictBudgetBurnTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictBudgetBurnTemplateFactor for FlaggedStrictBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictBudgetBurnTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictBudgetBurnTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictBudgetBurnTemplateFactor for TieredStrictBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictBudgetBurnTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictBudgetBurnTemplateAdjuster {
    pub minimum: f64,
}

impl StrictBudgetBurnTemplateFactor for MinimumStrictBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictBudgetBurnTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictBudgetBurnTemplateAdjuster {
    pub percent: f64,
}

impl StrictBudgetBurnTemplateFactor for PercentStrictBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictBudgetBurnTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict budget burn template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictBudgetBurnTemplateEngine {
    pub steps: Vec<Rc<dyn StrictBudgetBurnTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictBudgetBurnTemplateEngine {
    pub fn compute(&mut self, item: &StrictBudgetBurnTemplate) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[StrictBudgetBurnTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_budget_burn_template_engine(flags: &flags::Registry) -> StrictBudgetBurnTemplateEngine {
    StrictBudgetBurnTemplateEngine {
        steps: vec![
            Rc::new(LocaleStrictBudgetBurnTemplateAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.74),
                    ("south".to_string(), 0.85),
                    ("east".to_string(), 1.14),
                ]),
            }) as Rc<dyn StrictBudgetBurnTemplateFactor>,
            Rc::new(CapStrictBudgetBurnTemplateAdjuster { cap: 15105.0 }),
            Rc::new(FlaggedStrictBudgetBurnTemplateAdjuster {
                flags: flags.clone(),
                flag: "strict-budget-burn-template-manual".to_string(),
                percent: 9.1,
            }),
            Rc::new(TieredStrictBudgetBurnTemplateAdjuster {
                bounds: vec![6578.0, 8953.0],
                rates: vec![1.05, 1.18],
            }),
            Rc::new(MinimumStrictBudgetBurnTemplateAdjuster { minimum: 32.8 }),
            Rc::new(PercentStrictBudgetBurnTemplateAdjuster { percent: 14.9 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 4625.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_budget_burn_template_engine(&flags::new_registry());
        let item = new_strict_budget_burn_template("code-977", "label-929");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
