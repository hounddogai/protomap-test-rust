use super::*;

use acme_kit::text;

/// How screens and exports show a alert estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertEstimateView {
    pub code_text: String,
    pub title: String,
    pub currency: String,
    pub archived: String,
    pub namespace: String,
    pub weight: String,
    pub summary: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_alert_estimate_view(item: &AlertEstimate) -> AlertEstimateView {
    AlertEstimateView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        currency: item.currency.clone(),
        archived: format!("{}", item.archived),
        namespace: text::truncate(&item.namespace, 40),
        weight: format!("{:.2}", item.weight),
        summary: text::truncate(&item.summary, 80),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_estimate_views(items: &[AlertEstimate]) -> Vec<AlertEstimateView> {
    let mut views: Vec<AlertEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_estimate_view(item));
    }
    views
}

pub fn sum_alert_estimate_view(view: &AlertEstimateView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_alert_estimate("code-996", "title-402");
        let view = to_alert_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_alert_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
