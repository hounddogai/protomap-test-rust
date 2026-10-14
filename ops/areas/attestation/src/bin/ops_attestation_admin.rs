//! Command attestationctl runs attestation's jobs against sample data.

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_attestation::evidence::cycle as evidence_cycle;
use acme_attestation::measurement::notice as measurement_notice;
use acme_attestation::verifier::budget as verifier_budget;
use acme_kit::clock;
use acme_kit::flags;

fn main() {
    let clock: Rc<dyn clock::Clock> = clock::system();
    let commands: BTreeMap<String, Box<dyn Fn() -> bool + '_>> = BTreeMap::from([
        (
            "evidence-review".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn evidence_cycle::EvidenceReviewStore> =
                    evidence_cycle::new_memory_evidence_review_store();
                for index in 0..9 {
                    store.save(&evidence_cycle::new_evidence_review(
                        &format!("evidence-review-{}", index),
                        &format!("Sample evidence review {}", index),
                    ));
                }
                let mut flow = evidence_cycle::new_evidence_review_workflow(store.clone(), clock.clone());
                if flow.advance("evidence-review-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report = evidence_cycle::build_evidence_review_report("Evidence Review", &items, clock.clone());
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
            "measurement-notice".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn measurement_notice::MeasurementNoticeStore> =
                    measurement_notice::new_memory_measurement_notice_store();
                for index in 0..7 {
                    store.save(&measurement_notice::new_measurement_notice(
                        &format!("measurement-notice-{}", index),
                        &format!("Sample measurement notice {}", index),
                    ));
                }
                let mut flow = measurement_notice::new_measurement_notice_desk(
                    store.clone(),
                    clock.clone(),
                    &flags::new_registry(),
                );
                if flow.open("measurement-notice-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report =
                    measurement_notice::build_measurement_notice_report("Measurement Notice", &items, clock.clone());
                println!("{}", report.render());
                flow.audit(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|elapsed| elapsed.as_secs() as i64)
                        .unwrap_or_default(),
                ) >= 0
            }),
        ),
        (
            "verifier-segment".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn verifier_budget::VerifierSegmentStore> =
                    verifier_budget::new_memory_verifier_segment_store();
                for index in 0..5 {
                    store.save(&verifier_budget::new_verifier_segment(
                        &format!("verifier-segment-{}", index),
                        &format!("Sample verifier segment {}", index),
                    ));
                }
                let mut flow =
                    verifier_budget::new_verifier_segment_service(store.clone(), clock.clone(), &flags::new_registry());
                if flow.open("verifier-segment-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report = verifier_budget::build_verifier_segment_report("Verifier Segment", &items, clock.clone());
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
