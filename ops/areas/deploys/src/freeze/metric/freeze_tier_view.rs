use super::*;

use acme_kit::text;

/// How screens and exports show a freeze tier.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTierView {
    pub code: String,
    pub title: String,
    pub rotation_days: String,
    pub notes: String,
    pub window_days: String,
    pub margin_text: String,
    pub budget_text: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_freeze_tier_view(item: &FreezeTier) -> FreezeTierView {
    FreezeTierView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        rotation_days: item.rotation_days.to_string(),
        notes: item.notes.clone(),
        window_days: item.window_days.to_string(),
        margin_text: format!("{:.2}", item.margin),
        budget_text: format!("{:.2}", item.budget),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_tier_views(items: &[FreezeTier]) -> Vec<FreezeTierView> {
    let mut views: Vec<FreezeTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_tier_view(item));
    }
    views
}

pub fn sum_freeze_tier_view(view: &FreezeTierView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_window_days as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_tier("code-834", "title-120");
        let view = to_freeze_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
