use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemEstimateView {
    pub code_text: String,
    pub title_text: String,
    pub approved: String,
    pub locale: String,
    pub weight: String,
    pub cost: String,
    pub external_ref: String,
    pub expires_at: String,
    pub description_text: String,
    pub phone_text: String,
    pub status_label: String,
}

pub fn to_postmortem_estimate_view(item: &PostmortemEstimate) -> PostmortemEstimateView {
    PostmortemEstimateView {
        code_text: text::truncate(&item.code, 64),
        title_text: text::truncate(&item.title, 120),
        approved: format!("{}", item.approved),
        locale: item.locale.clone(),
        weight: format!("{:.2}", item.weight),
        cost: format!("{:.2}", item.cost),
        external_ref: item.external_ref.clone(),
        expires_at: item.expires_at.to_string(),
        description_text: item.description.clone(),
        phone_text: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_estimate_views(items: &[PostmortemEstimate]) -> Vec<PostmortemEstimateView> {
    let mut views: Vec<PostmortemEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_estimate_view(item));
    }
    views
}

pub fn sum_postmortem_estimate_view(view: &PostmortemEstimateView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_weight + parsed_cost + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_postmortem_estimate("code-892", "title-775");
        let view = to_postmortem_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_postmortem_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
