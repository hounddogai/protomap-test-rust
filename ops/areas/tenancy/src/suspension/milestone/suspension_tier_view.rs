use super::*;

use acme_kit::text;

/// How screens and exports show a suspension tier.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTierView {
    pub code: String,
    pub label: String,
    pub sequence: String,
    pub margin_text: String,
    pub window_days: String,
    pub budget: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_suspension_tier_view(item: &SuspensionTier) -> SuspensionTierView {
    SuspensionTierView {
        code: item.code.clone(),
        label: item.label.clone(),
        sequence: item.sequence.to_string(),
        margin_text: format!("{:.2}", item.margin),
        window_days: item.window_days.to_string(),
        budget: format!("{:.2}", item.budget),
        issuer: text::truncate(&item.issuer, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_tier_views(items: &[SuspensionTier]) -> Vec<SuspensionTierView> {
    let mut views: Vec<SuspensionTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_tier_view(item));
    }
    views
}

pub fn sum_suspension_tier_view(view: &SuspensionTierView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_margin + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_tier("code-119", "label-756");
        let view = to_suspension_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
