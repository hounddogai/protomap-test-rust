//! Command deploysctl runs deploys's jobs against sample data.

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_deploys::canary::checkpoint as canary_checkpoint;
use acme_deploys::release::slot as release_slot;
use acme_kit::clock;
use acme_kit::flags;

fn main() {
    let clock: Rc<dyn clock::Clock> = clock::system();
    let commands: BTreeMap<String, Box<dyn Fn() -> bool + '_>> = BTreeMap::from([
        (
            "release-slot".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn release_slot::ReleaseSlotStore> = release_slot::new_memory_release_slot_store();
                for index in 0..10 {
                    store.save(&release_slot::new_release_slot(
                        &format!("release-slot-{}", index),
                        &format!("Sample release slot {}", index),
                    ));
                }
                let mut flow =
                    release_slot::new_release_slot_desk(store.clone(), clock.clone(), &flags::new_registry());
                if flow.start("release-slot-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report = release_slot::build_release_slot_report("Release Slot", &items, clock.clone());
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
            "canary-override".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn canary_checkpoint::CanaryOverrideStore> =
                    canary_checkpoint::new_memory_canary_override_store();
                for index in 0..8 {
                    store.save(&canary_checkpoint::new_canary_override(
                        &format!("canary-override-{}", index),
                        &format!("Sample canary override {}", index),
                    ));
                }
                let mut flow = canary_checkpoint::new_canary_override_workflow(
                    store.clone(),
                    clock.clone(),
                    &flags::new_registry(),
                );
                if flow.submit("canary-override-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report = canary_checkpoint::build_canary_override_report("Canary Override", &items, clock.clone());
                println!("{}", report.render());
                flow.review_stale(
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
