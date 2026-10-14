use super::*;

use acme_kit::text;

/// How screens and exports show a manual redaction variance.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionVarianceView {
    pub code: String,
    pub name: String,
    pub quantity: String,
    pub window_days_text: String,
    pub budget: String,
    pub score_text: String,
    pub weight: String,
    pub host: String,
    pub rate: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_manual_redaction_variance_view(item: &ManualRedactionVariance) -> ManualRedactionVarianceView {
    ManualRedactionVarianceView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        quantity: item.quantity.to_string(),
        window_days_text: item.window_days.to_string(),
        budget: format!("{:.2}", item.budget),
        score_text: format!("{:.2}", item.score),
        weight: format!("{:.2}", item.weight),
        host: text::truncate(&item.host, 64),
        rate: format!("{:.2}", item.rate),
        notes: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_redaction_variance_views(items: &[ManualRedactionVariance]) -> Vec<ManualRedactionVarianceView> {
    let mut views: Vec<ManualRedactionVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_redaction_variance_view(item));
    }
    views
}

pub fn sum_manual_redaction_variance_view(view: &ManualRedactionVarianceView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_window_days as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_redaction_variance("code-938", "name-702");
        let view = to_manual_redaction_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_redaction_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
