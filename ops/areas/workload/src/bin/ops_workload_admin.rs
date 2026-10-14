//! Command workloadctl runs workload's jobs against sample data.

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;
use acme_workload::binding::group as binding_group;
use acme_workload::identity::manifest as identity_manifest;
use acme_workload::token::quota as token_quota;
use acme_workload::trust_domain::tier as trust_domain_tier;

fn main() {
    let clock: Rc<dyn clock::Clock> = clock::system();
    let commands: BTreeMap<String, Box<dyn Fn() -> bool + '_>> = BTreeMap::from([
        (
            "identity-manifest".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn identity_manifest::IdentityManifestStore> =
                    identity_manifest::new_memory_identity_manifest_store();
                for index in 0..12 {
                    store.save(&identity_manifest::new_identity_manifest(
                        &format!("identity-manifest-{}", index),
                        &format!("Sample identity manifest {}", index),
                    ));
                }
                let mut flow = identity_manifest::new_identity_manifest_service(store.clone(), clock.clone());
                if flow.open("identity-manifest-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report =
                    identity_manifest::build_identity_manifest_report("Identity Manifest", &items, clock.clone());
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
            "binding-tier".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn binding_group::BindingTierStore> = binding_group::new_memory_binding_tier_store();
                for index in 0..11 {
                    store.save(&binding_group::new_binding_tier(
                        &format!("binding-tier-{}", index),
                        &format!("Sample binding tier {}", index),
                    ));
                }
                let mut flow =
                    binding_group::new_binding_tier_service(store.clone(), clock.clone(), &flags::new_registry());
                if flow.submit("binding-tier-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report = binding_group::build_binding_tier_report("Binding Tier", &items, clock.clone());
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
            "token-override".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn token_quota::TokenOverrideStore> = token_quota::new_memory_token_override_store();
                for index in 0..12 {
                    store.save(&token_quota::new_token_override(
                        &format!("token-override-{}", index),
                        &format!("Sample token override {}", index),
                    ));
                }
                let mut flow = token_quota::new_token_override_workflow(store.clone(), clock.clone());
                if flow.open("token-override-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report = token_quota::build_token_override_report("Token Override", &items, clock.clone());
                println!("{}", report.render());
                flow.review_stale(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|elapsed| elapsed.as_secs() as i64)
                        .unwrap_or_default(),
                ) >= 0
            }),
        ),
        (
            "trust-domain-tier".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn trust_domain_tier::TrustDomainTierStore> =
                    trust_domain_tier::new_memory_trust_domain_tier_store();
                for index in 0..8 {
                    store.save(&trust_domain_tier::new_trust_domain_tier(
                        &format!("trust-domain-tier-{}", index),
                        &format!("Sample trust domain tier {}", index),
                    ));
                }
                let mut flow = trust_domain_tier::new_trust_domain_tier_desk(store.clone(), clock.clone());
                if flow.open("trust-domain-tier-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report =
                    trust_domain_tier::build_trust_domain_tier_report("Trust Domain Tier", &items, clock.clone());
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
