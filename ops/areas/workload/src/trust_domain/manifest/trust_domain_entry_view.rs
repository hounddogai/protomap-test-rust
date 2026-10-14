use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain entry.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainEntryView {
    pub code: String,
    pub name_text: String,
    pub fingerprint: String,
    pub reviewer: String,
    pub port: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_trust_domain_entry_view(item: &TrustDomainEntry) -> TrustDomainEntryView {
    TrustDomainEntryView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        fingerprint: item.fingerprint.clone(),
        reviewer: item.reviewer.clone(),
        port: item.port.to_string(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_entry_views(items: &[TrustDomainEntry]) -> Vec<TrustDomainEntryView> {
    let mut views: Vec<TrustDomainEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_entry_view(item));
    }
    views
}

pub fn sum_trust_domain_entry_view(view: &TrustDomainEntryView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_port as f64 + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_entry("code-207", "name-688");
        let view = to_trust_domain_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
