use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyEstimateView {
    pub code_text: String,
    pub title_text: String,
    pub notes: String,
    pub discount: String,
    pub locale: String,
    pub issuer: String,
    pub approved_text: String,
    pub archived: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_ttl_policy_estimate_view(item: &TtlPolicyEstimate) -> TtlPolicyEstimateView {
    TtlPolicyEstimateView {
        code_text: text::truncate(&item.code, 64),
        title_text: text::truncate(&item.title, 64),
        notes: item.notes.clone(),
        discount: format!("{:.2}", item.discount),
        locale: item.locale.clone(),
        issuer: item.issuer.clone(),
        approved_text: format!("{}", item.approved),
        archived: format!("{}", item.archived),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_estimate_views(items: &[TtlPolicyEstimate]) -> Vec<TtlPolicyEstimateView> {
    let mut views: Vec<TtlPolicyEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_estimate_view(item));
    }
    views
}

pub fn sum_ttl_policy_estimate_view(view: &TtlPolicyEstimateView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_estimate("code-815", "title-278");
        let view = to_ttl_policy_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
