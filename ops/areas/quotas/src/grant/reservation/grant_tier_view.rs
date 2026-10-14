use super::*;

use acme_kit::text;

/// How screens and exports show a grant tier.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantTierView {
    pub code_text: String,
    pub title_text: String,
    pub retries: String,
    pub phone: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_grant_tier_view(item: &GrantTier) -> GrantTierView {
    GrantTierView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        retries: item.retries.to_string(),
        phone: text::truncate(&item.phone, 64),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_tier_views(items: &[GrantTier]) -> Vec<GrantTierView> {
    let mut views: Vec<GrantTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_tier_view(item));
    }
    views
}

pub fn sum_grant_tier_view(view: &GrantTierView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_tier("code-996", "title-661");
        let view = to_grant_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
