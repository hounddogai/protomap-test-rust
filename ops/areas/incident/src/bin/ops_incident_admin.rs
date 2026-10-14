//! Command incidentctl runs incident's jobs against sample data.

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_incident::escalation::policy as escalation_policy;
use acme_incident::page::rule as page_rule;
use acme_incident::timeline::exception as timeline_exception;
use acme_kit::clock;
use acme_kit::flags;

fn main() {
    let clock: Rc<dyn clock::Clock> = clock::system();
    let commands: BTreeMap<String, Box<dyn Fn() -> bool + '_>> = BTreeMap::from([
        (
            "page-template".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn page_rule::PageTemplateStore> = page_rule::new_memory_page_template_store();
                for index in 0..12 {
                    store.save(&page_rule::new_page_template(
                        &format!("page-template-{}", index),
                        &format!("Sample page template {}", index),
                    ));
                }
                let mut flow = page_rule::new_page_template_workflow(store.clone(), clock.clone());
                if flow.open("page-template-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report = page_rule::build_page_template_report("Page Template", &items, clock.clone());
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
            "escalation-metric".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn escalation_policy::EscalationMetricStore> =
                    escalation_policy::new_memory_escalation_metric_store();
                for index in 0..4 {
                    store.save(&escalation_policy::new_escalation_metric(
                        &format!("escalation-metric-{}", index),
                        &format!("Sample escalation metric {}", index),
                    ));
                }
                let mut flow = escalation_policy::new_escalation_metric_manager(
                    store.clone(),
                    clock.clone(),
                    &flags::new_registry(),
                );
                if flow.advance("escalation-metric-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report =
                    escalation_policy::build_escalation_metric_report("Escalation Metric", &items, clock.clone());
                println!("{}", report.render());
                flow.scan(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|elapsed| elapsed.as_secs() as i64)
                        .unwrap_or_default(),
                ) >= 0
            }),
        ),
        (
            "timeline-exception".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn timeline_exception::TimelineExceptionStore> =
                    timeline_exception::new_memory_timeline_exception_store();
                for index in 0..5 {
                    store.save(&timeline_exception::new_timeline_exception(
                        &format!("timeline-exception-{}", index),
                        &format!("Sample timeline exception {}", index),
                    ));
                }
                let mut flow = timeline_exception::new_timeline_exception_service(
                    store.clone(),
                    clock.clone(),
                    &flags::new_registry(),
                );
                if flow.open("timeline-exception-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report =
                    timeline_exception::build_timeline_exception_report("Timeline Exception", &items, clock.clone());
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
