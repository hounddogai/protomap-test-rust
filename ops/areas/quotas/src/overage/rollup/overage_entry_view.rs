use super::*;

use acme_kit::text;

/// How screens and exports show a overage entry.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageEntryView {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub pinned: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_overage_entry_view(item: &OverageEntry) -> OverageEntryView {
    OverageEntryView {
        code: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 120),
        external_ref: item.external_ref.clone(),
        pinned: format!("{}", item.pinned),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_entry_views(items: &[OverageEntry]) -> Vec<OverageEntryView> {
    let mut views: Vec<OverageEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_entry_view(item));
    }
    views
}

pub fn sum_overage_entry_view(view: &OverageEntryView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_port as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_entry("code-131", "name-176");
        let view = to_overage_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
