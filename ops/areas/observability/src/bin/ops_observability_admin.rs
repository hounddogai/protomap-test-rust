//! Command observabilityctl runs observability's jobs against sample data.

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;
use acme_observability::budget_burn::template as budget_burn_template;
use acme_observability::slo::targets as slo_targets;

fn main() {
    let clock: Rc<dyn clock::Clock> = clock::system();
    let commands: BTreeMap<String, Box<dyn Fn() -> bool + '_>> = BTreeMap::from([
        (
            "slo-metric".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn slo_targets::SloMetricStore> = slo_targets::new_memory_slo_metric_store();
                for index in 0..8 {
                    store.save(&slo_targets::new_slo_metric(
                        &format!("slo-metric-{}", index),
                        &format!("Sample slo metric {}", index),
                    ));
                }
                let mut flow =
                    slo_targets::new_slo_metric_manager(store.clone(), clock.clone(), &flags::new_registry());
                if flow.start("slo-metric-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report = slo_targets::build_slo_metric_report("Slo Metric", &items, clock.clone());
                println!("{}", report.render());
                flow.audit(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|elapsed| elapsed.as_secs() as i64)
                        .unwrap_or_default(),
                ) >= 0
            }) as Box<dyn Fn() -> bool + '_>,
        ),
        (
            "regional-budget-burn-template".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn budget_burn_template::RegionalBudgetBurnTemplateStore> =
                    budget_burn_template::new_memory_regional_budget_burn_template_store();
                for index in 0..8 {
                    store.save(&budget_burn_template::new_regional_budget_burn_template(
                        &format!("regional-budget-burn-template-{}", index),
                        &format!("Sample regional budget burn template {}", index),
                    ));
                }
                let mut flow =
                    budget_burn_template::new_regional_budget_burn_template_coordinator(store.clone(), clock.clone());
                if flow.advance("regional-budget-burn-template-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report = budget_burn_template::build_regional_budget_burn_template_report(
                    "Regional Budget Burn Template",
                    &items,
                    clock.clone(),
                );
                println!("{}", report.render());
                flow.scan(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|elapsed| elapsed.as_secs() as i64)
                        .unwrap_or_default(),
                ) >= 0
            }),
        ),
    ]);
    let mut failed = 0;
    for (name, command) in &commands {
        if !command() {
            println!("{}", format!("{} failed", name));
            failed += 1;
        }
    }
    if failed > 0 {
        std::process::exit(1 as i32);
    }
}
