use super::*;

/// How screens and exports show a weighted rule variance.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleVarianceView {
    pub code: String,
    pub title_text: String,
    pub notes: String,
    pub currency_text: String,
    pub window_days: String,
    pub updated_at: String,
    pub phone: String,
    pub source: String,
    pub starts_at: String,
    pub burst: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_weighted_rule_variance_view(item: &WeightedRuleVariance) -> WeightedRuleVarianceView {
    WeightedRuleVarianceView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        notes: item.notes.clone(),
        currency_text: item.currency.clone(),
        window_days: item.window_days.to_string(),
        updated_at: item.updated_at.to_string(),
        phone: item.phone.clone(),
        source: item.source.clone(),
        starts_at: item.starts_at.to_string(),
        burst: item.burst.to_string(),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_rule_variance_views(items: &[WeightedRuleVariance]) -> Vec<WeightedRuleVarianceView> {
    let mut views: Vec<WeightedRuleVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_rule_variance_view(item));
    }
    views
}

pub fn sum_weighted_rule_variance_view(view: &WeightedRuleVarianceView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_updated_at as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_rule_variance("code-104", "title-754");
        let view = to_weighted_rule_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_rule_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
