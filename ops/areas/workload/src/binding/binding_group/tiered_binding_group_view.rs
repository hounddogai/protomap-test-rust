use super::*;

use acme_kit::text;

/// How screens and exports show a tiered binding group.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingGroupView {
    pub code: String,
    pub title: String,
    pub ratio: String,
    pub region: String,
    pub fingerprint: String,
    pub country: String,
    pub tenant_id: String,
    pub phone_text: String,
    pub sequence: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_tiered_binding_group_view(item: &TieredBindingGroup) -> TieredBindingGroupView {
    TieredBindingGroupView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 120),
        ratio: format!("{:.2}", item.ratio),
        region: text::truncate(&item.region, 120),
        fingerprint: item.fingerprint.clone(),
        country: item.country.clone(),
        tenant_id: text::truncate(&item.tenant_id, 40),
        phone_text: item.phone.clone(),
        sequence: item.sequence.to_string(),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_binding_group_views(items: &[TieredBindingGroup]) -> Vec<TieredBindingGroupView> {
    let mut views: Vec<TieredBindingGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_binding_group_view(item));
    }
    views
}

pub fn sum_tiered_binding_group_view(view: &TieredBindingGroupView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_ratio + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_binding_group("code-159", "title-706");
        let view = to_tiered_binding_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_binding_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
