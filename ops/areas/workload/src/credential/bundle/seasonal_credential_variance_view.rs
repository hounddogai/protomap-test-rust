use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal credential variance.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialVarianceView {
    pub code_text: String,
    pub title: String,
    pub window_days: String,
    pub reviewer_text: String,
    pub score_text: String,
    pub currency_text: String,
    pub timezone: String,
    pub scope: String,
    pub pinned: String,
    pub contact_email_text: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_seasonal_credential_variance_view(item: &SeasonalCredentialVariance) -> SeasonalCredentialVarianceView {
    SeasonalCredentialVarianceView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        window_days: item.window_days.to_string(),
        reviewer_text: item.reviewer.clone(),
        score_text: format!("{:.2}", item.score),
        currency_text: item.currency.clone(),
        timezone: item.timezone.clone(),
        scope: text::truncate(&item.scope, 120),
        pinned: format!("{}", item.pinned),
        contact_email_text: text::truncate(&item.contact_email, 80),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_credential_variance_views(
    items: &[SeasonalCredentialVariance],
) -> Vec<SeasonalCredentialVarianceView> {
    let mut views: Vec<SeasonalCredentialVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_credential_variance_view(item));
    }
    views
}

pub fn sum_seasonal_credential_variance_view(view: &SeasonalCredentialVarianceView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_score + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_credential_variance("code-944", "title-857");
        let view = to_seasonal_credential_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_credential_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
