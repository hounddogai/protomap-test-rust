//! Command tenancyctl runs tenancy's jobs against sample data.

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;
use acme_tenancy::plan::score as plan_score;
use acme_tenancy::tenant::ledger as tenant_ledger;

fn main() {
    let clock: Rc<dyn clock::Clock> = clock::system();
    let commands: BTreeMap<String, Box<dyn Fn() -> bool + '_>> = BTreeMap::from([
        (
            "tenant-allocation".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn tenant_ledger::TenantAllocationStore> =
                    tenant_ledger::new_memory_tenant_allocation_store();
                for index in 0..5 {
                    store.save(&tenant_ledger::new_tenant_allocation(
                        &format!("tenant-allocation-{}", index),
                        &format!("Sample tenant allocation {}", index),
                    ));
                }
                let mut flow =
                    tenant_ledger::new_tenant_allocation_service(store.clone(), clock.clone(), &flags::new_registry());
                if flow.start("tenant-allocation-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report = tenant_ledger::build_tenant_allocation_report("Tenant Allocation", &items, clock.clone());
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
            "plan-quota".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn plan_score::PlanQuotaStore> = plan_score::new_memory_plan_quota_store();
                for index in 0..7 {
                    store.save(&plan_score::new_plan_quota(
                        &format!("plan-quota-{}", index),
                        &format!("Sample plan quota {}", index),
                    ));
                }
                let mut flow =
                    plan_score::new_plan_quota_workflow(store.clone(), clock.clone(), &flags::new_registry());
                if flow.open("plan-quota-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report = plan_score::build_plan_quota_report("Plan Quota", &items, clock.clone());
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
