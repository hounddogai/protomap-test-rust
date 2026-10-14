//! Command quotasctl runs quotas's jobs against sample data.

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::clock;
use acme_quotas::limit::allocation as limit_allocation;

fn main() {
    let clock: Rc<dyn clock::Clock> = clock::system();
    let commands: BTreeMap<String, Box<dyn Fn() -> bool + '_>> = BTreeMap::from([(
        "limit-forecast".to_string(),
        Box::new(|| -> bool {
            let store: Rc<dyn limit_allocation::LimitForecastStore> =
                limit_allocation::new_memory_limit_forecast_store();
            for index in 0..12 {
                store.save(&limit_allocation::new_limit_forecast(
                    &format!("limit-forecast-{}", index),
                    &format!("Sample limit forecast {}", index),
                ));
            }
            let mut flow = limit_allocation::new_limit_forecast_desk(store.clone(), clock.clone());
            if flow.start("limit-forecast-0").is_err() {
                return false;
            }
            let items = store.list();
            let report = limit_allocation::build_limit_forecast_report("Limit Forecast", &items, clock.clone());
            println!("{}", report.render());
            flow.sweep(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|elapsed| elapsed.as_secs() as i64)
                    .unwrap_or_default(),
            ) >= 0
        }) as Box<dyn Fn() -> bool + '_>,
    )]);
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
