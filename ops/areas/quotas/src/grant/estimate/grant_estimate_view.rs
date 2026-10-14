use super::*;

/// How screens and exports show a grant estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantEstimateView {
    pub code: String,
    pub label: String,
    pub region_text: String,
    pub updated_at_text: String,
    pub port: String,
    pub notes_text: String,
    pub ends_at_text: String,
    pub country_text: String,
    pub score: String,
    pub source_text: String,
    pub cost_text: String,
    pub status_label: String,
}

pub fn to_grant_estimate_view(item: &GrantEstimate) -> GrantEstimateView {
    GrantEstimateView {
        code: item.code.clone(),
        label: item.label.clone(),
        region_text: item.region.clone(),
        updated_at_text: item.updated_at.to_string(),
        port: item.port.to_string(),
        notes_text: item.notes.clone(),
        ends_at_text: item.ends_at.to_string(),
        country_text: item.country.clone(),
        score: format!("{:.2}", item.score),
        source_text: item.source.clone(),
        cost_text: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_estimate_views(items: &[GrantEstimate]) -> Vec<GrantEstimateView> {
    let mut views: Vec<GrantEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_estimate_view(item));
    }
    views
}

pub fn sum_grant_estimate_view(view: &GrantEstimateView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_port as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_estimate("code-573", "label-764");
        let view = to_grant_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
