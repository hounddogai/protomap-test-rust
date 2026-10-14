use super::*;

use acme_kit::text;

/// How screens and exports show a default rollout variance.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutVarianceView {
    pub code: String,
    pub title: String,
    pub category: String,
    pub created_at: String,
    pub key_id: String,
    pub ratio_text: String,
    pub quantity_text: String,
    pub limit_text: String,
    pub external_ref: String,
    pub name: String,
    pub rate_text: String,
    pub status_label: String,
}

pub fn to_default_rollout_variance_view(item: &DefaultRolloutVariance) -> DefaultRolloutVarianceView {
    DefaultRolloutVarianceView {
        code: item.code.clone(),
        title: item.title.clone(),
        category: text::truncate(&item.category, 120),
        created_at: item.created_at.to_string(),
        key_id: item.key_id.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        quantity_text: item.quantity.to_string(),
        limit_text: item.limit.to_string(),
        external_ref: item.external_ref.clone(),
        name: item.name.clone(),
        rate_text: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_rollout_variance_views(items: &[DefaultRolloutVariance]) -> Vec<DefaultRolloutVarianceView> {
    let mut views: Vec<DefaultRolloutVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_rollout_variance_view(item));
    }
    views
}

pub fn sum_default_rollout_variance_view(view: &DefaultRolloutVarianceView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_ratio + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_rollout_variance("code-445", "title-501");
        let view = to_default_rollout_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_rollout_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
