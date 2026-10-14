use super::*;

use acme_kit::text;

/// How screens and exports show a overage profile.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageProfileView {
    pub code: String,
    pub label_text: String,
    pub region: String,
    pub sequence_text: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_overage_profile_view(item: &OverageProfile) -> OverageProfileView {
    OverageProfileView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        region: text::truncate(&item.region, 40),
        sequence_text: item.sequence.to_string(),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_profile_views(items: &[OverageProfile]) -> Vec<OverageProfileView> {
    let mut views: Vec<OverageProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_profile_view(item));
    }
    views
}

pub fn sum_overage_profile_view(view: &OverageProfileView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_sequence as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_profile("code-699", "label-54");
        let view = to_overage_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
