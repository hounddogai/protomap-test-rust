//! Command netpolicyctl runs netpolicy's jobs against sample data.

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;
use acme_netpolicy::allowlist::snapshot as allowlist_snapshot;
use acme_netpolicy::rule::exception as rule_exception;
use acme_netpolicy::segment::schedule as segment_schedule;

fn main() {
    let clock: Rc<dyn clock::Clock> = clock::system();
    let commands: BTreeMap<String, Box<dyn Fn() -> bool + '_>> = BTreeMap::from([
        (
            "rule-snapshot".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn rule_exception::RuleSnapshotStore> = rule_exception::new_memory_rule_snapshot_store();
                for index in 0..3 {
                    store.save(&rule_exception::new_rule_snapshot(
                        &format!("rule-snapshot-{}", index),
                        &format!("Sample rule snapshot {}", index),
                    ));
                }
                let mut flow =
                    rule_exception::new_rule_snapshot_coordinator(store.clone(), clock.clone(), &flags::new_registry());
                if flow.start("rule-snapshot-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report = rule_exception::build_rule_snapshot_report("Rule Snapshot", &items, clock.clone());
                println!("{}", report.render());
                flow.review_stale(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|elapsed| elapsed.as_secs() as i64)
                        .unwrap_or_default(),
                ) >= 0
            }) as Box<dyn Fn() -> bool + '_>,
        ),
        (
            "segment-manifest".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn segment_schedule::SegmentManifestStore> =
                    segment_schedule::new_memory_segment_manifest_store();
                for index in 0..4 {
                    store.save(&segment_schedule::new_segment_manifest(
                        &format!("segment-manifest-{}", index),
                        &format!("Sample segment manifest {}", index),
                    ));
                }
                let mut flow = segment_schedule::new_segment_manifest_workflow(store.clone(), clock.clone());
                if flow.submit("segment-manifest-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report = segment_schedule::build_segment_manifest_report("Segment Manifest", &items, clock.clone());
                println!("{}", report.render());
                flow.sweep(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|elapsed| elapsed.as_secs() as i64)
                        .unwrap_or_default(),
                ) >= 0
            }),
        ),
        (
            "allowlist-window".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn allowlist_snapshot::AllowlistWindowStore> =
                    allowlist_snapshot::new_memory_allowlist_window_store();
                for index in 0..4 {
                    store.save(&allowlist_snapshot::new_allowlist_window(
                        &format!("allowlist-window-{}", index),
                        &format!("Sample allowlist window {}", index),
                    ));
                }
                let mut flow = allowlist_snapshot::new_allowlist_window_manager(store.clone(), clock.clone());
                if flow.start("allowlist-window-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report =
                    allowlist_snapshot::build_allowlist_window_report("Allowlist Window", &items, clock.clone());
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
