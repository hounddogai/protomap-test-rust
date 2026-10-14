//! Command dnsctl runs dns's jobs against sample data.

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_dns::dns_record::preference as dns_record_preference;
use acme_dns::zone::quota as zone_quota;
use acme_kit::clock;

fn main() {
    let clock: Rc<dyn clock::Clock> = clock::system();
    let commands: BTreeMap<String, Box<dyn Fn() -> bool + '_>> = BTreeMap::from([
        (
            "zone-override".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn zone_quota::ZoneOverrideStore> = zone_quota::new_memory_zone_override_store();
                for index in 0..11 {
                    store.save(&zone_quota::new_zone_override(
                        &format!("zone-override-{}", index),
                        &format!("Sample zone override {}", index),
                    ));
                }
                let mut flow = zone_quota::new_zone_override_manager(store.clone(), clock.clone());
                if flow.submit("zone-override-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report = zone_quota::build_zone_override_report("Zone Override", &items, clock.clone());
                println!("{}", report.render());
                flow.scan(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|elapsed| elapsed.as_secs() as i64)
                        .unwrap_or_default(),
                ) >= 0
            }) as Box<dyn Fn() -> bool + '_>,
        ),
        (
            "dns-record-profile".to_string(),
            Box::new(|| -> bool {
                let store: Rc<dyn dns_record_preference::DnsRecordProfileStore> =
                    dns_record_preference::new_memory_dns_record_profile_store();
                for index in 0..9 {
                    store.save(&dns_record_preference::new_dns_record_profile(
                        &format!("dns-record-profile-{}", index),
                        &format!("Sample dns record profile {}", index),
                    ));
                }
                let mut flow = dns_record_preference::new_dns_record_profile_manager(store.clone(), clock.clone());
                if flow.advance("dns-record-profile-0").is_err() {
                    return false;
                }
                let items = store.list();
                let report =
                    dns_record_preference::build_dns_record_profile_report("Dns Record Profile", &items, clock.clone());
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
