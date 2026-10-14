use super::*;

use acme_kit::text;

/// How screens and exports show a quote bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBundleView {
    pub code: String,
    pub title: String,
    pub amount: String,
    pub key_id: String,
    pub principal: String,
    pub active: String,
    pub rotation_days: String,
    pub duration_minutes: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_quote_bundle_view(item: &QuoteBundle) -> QuoteBundleView {
    QuoteBundleView {
        code: item.code.clone(),
        title: item.title.clone(),
        amount: format!("{:.2}", item.amount),
        key_id: text::truncate(&item.key_id, 80),
        principal: item.principal.clone(),
        active: format!("{}", item.active),
        rotation_days: item.rotation_days.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_bundle_views(items: &[QuoteBundle]) -> Vec<QuoteBundleView> {
    let mut views: Vec<QuoteBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_bundle_view(item));
    }
    views
}

pub fn sum_quote_bundle_view(view: &QuoteBundleView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_amount + (parsed_rotation_days as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_bundle("code-209", "title-491");
        let view = to_quote_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
