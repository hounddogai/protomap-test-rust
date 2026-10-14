use super::*;

use acme_kit::text;

/// How screens and exports show a default plan entry.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanEntryView {
    pub code_text: String,
    pub title: String,
    pub threshold: String,
    pub host: String,
    pub tenant_id: String,
    pub pinned: String,
    pub checksum: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_default_plan_entry_view(item: &DefaultPlanEntry) -> DefaultPlanEntryView {
    DefaultPlanEntryView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        threshold: format!("{:.2}", item.threshold),
        host: text::truncate(&item.host, 40),
        tenant_id: text::truncate(&item.tenant_id, 120),
        pinned: format!("{}", item.pinned),
        checksum: item.checksum.clone(),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_plan_entry_views(items: &[DefaultPlanEntry]) -> Vec<DefaultPlanEntryView> {
    let mut views: Vec<DefaultPlanEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_plan_entry_view(item));
    }
    views
}

pub fn sum_default_plan_entry_view(view: &DefaultPlanEntryView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_plan_entry("code-284", "title-871");
        let view = to_default_plan_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_plan_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
