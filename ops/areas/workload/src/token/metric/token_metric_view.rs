use super::*;

/// How screens and exports show a token metric.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenMetricView {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub name_text: String,
    pub budget: String,
    pub updated_at: String,
    pub maximum: String,
    pub duration_minutes_text: String,
    pub currency_text: String,
    pub status_label: String,
}

pub fn to_token_metric_view(item: &TokenMetric) -> TokenMetricView {
    TokenMetricView {
        code: item.code.clone(),
        label: item.label.clone(),
        cipher: item.cipher.clone(),
        name_text: item.name.clone(),
        budget: format!("{:.2}", item.budget),
        updated_at: item.updated_at.to_string(),
        maximum: format!("{:.2}", item.maximum),
        duration_minutes_text: item.duration_minutes.to_string(),
        currency_text: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_metric_views(items: &[TokenMetric]) -> Vec<TokenMetricView> {
    let mut views: Vec<TokenMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_metric_view(item));
    }
    views
}

pub fn sum_token_metric_view(view: &TokenMetricView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_budget + (parsed_updated_at as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_metric("code-247", "label-537");
        let view = to_token_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
