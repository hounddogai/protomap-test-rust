use super::*;

use acme_kit::text;

/// How screens and exports show a pending federation variance.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationVarianceView {
    pub code: String,
    pub label: String,
    pub description_text: String,
    pub cipher: String,
    pub rate: String,
    pub minimum: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_pending_federation_variance_view(item: &PendingFederationVariance) -> PendingFederationVarianceView {
    PendingFederationVarianceView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        description_text: item.description.clone(),
        cipher: item.cipher.clone(),
        rate: format!("{:.2}", item.rate),
        minimum: format!("{:.2}", item.minimum),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_federation_variance_views(items: &[PendingFederationVariance]) -> Vec<PendingFederationVarianceView> {
    let mut views: Vec<PendingFederationVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_federation_variance_view(item));
    }
    views
}

pub fn sum_pending_federation_variance_view(view: &PendingFederationVarianceView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_rate + parsed_minimum + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_federation_variance("code-212", "label-770");
        let view = to_pending_federation_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_federation_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
