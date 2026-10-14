use super::*;

use acme_kit::text;

/// How screens and exports show a manual delegation score.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDelegationScoreView {
    pub code: String,
    pub name_text: String,
    pub created_at: String,
    pub scope: String,
    pub ceiling: String,
    pub limit: String,
    pub contact_email: String,
    pub ends_at_text: String,
    pub description_text: String,
    pub channel_text: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_manual_delegation_score_view(item: &ManualDelegationScore) -> ManualDelegationScoreView {
    ManualDelegationScoreView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        created_at: item.created_at.to_string(),
        scope: text::truncate(&item.scope, 120),
        ceiling: format!("{:.2}", item.ceiling),
        limit: item.limit.to_string(),
        contact_email: item.contact_email.clone(),
        ends_at_text: item.ends_at.to_string(),
        description_text: text::truncate(&item.description, 120),
        channel_text: item.channel.clone(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_delegation_score_views(items: &[ManualDelegationScore]) -> Vec<ManualDelegationScoreView> {
    let mut views: Vec<ManualDelegationScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_delegation_score_view(item));
    }
    views
}

pub fn sum_manual_delegation_score_view(view: &ManualDelegationScoreView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_ceiling + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_delegation_score("code-437", "name-270");
        let view = to_manual_delegation_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_delegation_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
