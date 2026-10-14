use super::*;

use acme_kit::text;

/// How screens and exports show a legacy peering exception.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPeeringExceptionView {
    pub code: String,
    pub label_text: String,
    pub approved_text: String,
    pub priority_text: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_legacy_peering_exception_view(item: &LegacyPeeringException) -> LegacyPeeringExceptionView {
    LegacyPeeringExceptionView {
        code: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        approved_text: format!("{}", item.approved),
        priority_text: item.priority.to_string(),
        key_id: text::truncate(&item.key_id, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_peering_exception_views(items: &[LegacyPeeringException]) -> Vec<LegacyPeeringExceptionView> {
    let mut views: Vec<LegacyPeeringExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_peering_exception_view(item));
    }
    views
}

pub fn sum_legacy_peering_exception_view(view: &LegacyPeeringExceptionView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_peering_exception("code-558", "label-196");
        let view = to_legacy_peering_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_peering_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
