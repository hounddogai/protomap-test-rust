use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled token variance.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTokenVarianceView {
    pub code: String,
    pub label_text: String,
    pub burst_text: String,
    pub discount: String,
    pub key_id_text: String,
    pub status_label: String,
}

pub fn to_scheduled_token_variance_view(item: &ScheduledTokenVariance) -> ScheduledTokenVarianceView {
    ScheduledTokenVarianceView {
        code: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        burst_text: item.burst.to_string(),
        discount: format!("{:.2}", item.discount),
        key_id_text: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_token_variance_views(items: &[ScheduledTokenVariance]) -> Vec<ScheduledTokenVarianceView> {
    let mut views: Vec<ScheduledTokenVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_token_variance_view(item));
    }
    views
}

pub fn sum_scheduled_token_variance_view(view: &ScheduledTokenVarianceView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_burst as f64 + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_token_variance("code-449", "label-664");
        let view = to_scheduled_token_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_token_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
