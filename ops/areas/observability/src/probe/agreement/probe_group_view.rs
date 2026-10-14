use super::*;

use acme_kit::text;

/// How screens and exports show a probe group.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeGroupView {
    pub code: String,
    pub label_text: String,
    pub fingerprint_text: String,
    pub archived: String,
    pub principal_text: String,
    pub approved: String,
    pub description: String,
    pub segment_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_probe_group_view(item: &ProbeGroup) -> ProbeGroupView {
    ProbeGroupView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        fingerprint_text: item.fingerprint.clone(),
        archived: format!("{}", item.archived),
        principal_text: item.principal.clone(),
        approved: format!("{}", item.approved),
        description: item.description.clone(),
        segment_text: text::truncate(&item.segment, 80),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_group_views(items: &[ProbeGroup]) -> Vec<ProbeGroupView> {
    let mut views: Vec<ProbeGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_group_view(item));
    }
    views
}

pub fn sum_probe_group_view(view: &ProbeGroupView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_group("code-452", "label-161");
        let view = to_probe_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
