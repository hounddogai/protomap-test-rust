//! Command audittrailctl runs audittrail's jobs against sample data.

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_audittrail::retention::slot as retention_slot;
use acme_audittrail::sink::quota as sink_quota;
use acme_kit::clock;
use acme_kit::flags;

fn main() {
    let clock: Rc<dyn clock::Clock> = clock::system();
    let commands: BTreeMap<String, Box<dyn Fn() -> bool + '_>> = BTreeMap::from([
        (
            "sink-threshold".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn sink_quota::SinkThresholdStore> = sink_quota::new_memory_sink_threshold_store();
                for index in 0..12 {
                    store.save(&sink_quota::new_sink_threshold(
                        &format!("sink-threshold-{}", index),
                        &format!("Sample sink threshold {}", index),
                    ));
                }
                let mut flow = sink_quota::new_sink_threshold_coordinator(store.clone(), clock.clone());
                if flow.start("sink-threshold-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report = sink_quota::build_sink_threshold_report("Sink Threshold", &items, clock.clone());
                println!("{}", report.render());
                flow.sweep(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|elapsed| elapsed.as_secs() as i64)
                        .unwrap_or_default(),
                ) >= 0
            }) as Box<dyn Fn() -> bool + '_>,
        ),
        (
            "retention-mapping".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn retention_slot::RetentionMappingStore> =
                    retention_slot::new_memory_retention_mapping_store();
                for index in 0..8 {
                    store.save(&retention_slot::new_retention_mapping(
                        &format!("retention-mapping-{}", index),
                        &format!("Sample retention mapping {}", index),
                    ));
                }
                let mut flow =
                    retention_slot::new_retention_mapping_service(store.clone(), clock.clone(), &flags::new_registry());
                if flow.start("retention-mapping-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report = retention_slot::build_retention_mapping_report("Retention Mapping", &items, clock.clone());
                println!("{}", report.render());
                flow.sweep(
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
