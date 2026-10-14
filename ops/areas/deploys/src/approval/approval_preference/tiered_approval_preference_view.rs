use super::*;

use acme_kit::text;

/// How screens and exports show a tiered approval preference.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalPreferenceView {
    pub code_text: String,
    pub title_text: String,
    pub rate: String,
    pub issuer: String,
    pub channel: String,
    pub rotation_days: String,
    pub updated_at: String,
    pub score: String,
    pub scope: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_tiered_approval_preference_view(item: &TieredApprovalPreference) -> TieredApprovalPreferenceView {
    TieredApprovalPreferenceView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        rate: format!("{:.2}", item.rate),
        issuer: item.issuer.clone(),
        channel: item.channel.clone(),
        rotation_days: item.rotation_days.to_string(),
        updated_at: item.updated_at.to_string(),
        score: format!("{:.2}", item.score),
        scope: text::truncate(&item.scope, 40),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_approval_preference_views(items: &[TieredApprovalPreference]) -> Vec<TieredApprovalPreferenceView> {
    let mut views: Vec<TieredApprovalPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_approval_preference_view(item));
    }
    views
}

pub fn sum_tiered_approval_preference_view(view: &TieredApprovalPreferenceView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_rate + (parsed_rotation_days as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_approval_preference("code-135", "title-643");
        let view = to_tiered_approval_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_approval_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
