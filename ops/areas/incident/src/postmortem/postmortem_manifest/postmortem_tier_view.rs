use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem tier.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemTierView {
    pub code: String,
    pub label: String,
    pub window_days: String,
    pub created_at: String,
    pub country_text: String,
    pub audience: String,
    pub ends_at_text: String,
    pub status_label: String,
}

pub fn to_postmortem_tier_view(item: &PostmortemTier) -> PostmortemTierView {
    PostmortemTierView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        window_days: item.window_days.to_string(),
        created_at: item.created_at.to_string(),
        country_text: text::truncate(&item.country, 80),
        audience: item.audience.clone(),
        ends_at_text: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_tier_views(items: &[PostmortemTier]) -> Vec<PostmortemTierView> {
    let mut views: Vec<PostmortemTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_tier_view(item));
    }
    views
}

pub fn sum_postmortem_tier_view(view: &PostmortemTierView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_created_at as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_postmortem_tier("code-231", "label-78");
        let view = to_postmortem_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_postmortem_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
