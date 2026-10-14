use super::*;

use acme_kit::text;

/// How screens and exports show a regional failover variance.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverVarianceView {
    pub code_text: String,
    pub name: String,
    pub quantity_text: String,
    pub created_at: String,
    pub window_days: String,
    pub margin_text: String,
    pub scope: String,
    pub ceiling_text: String,
    pub title: String,
    pub cluster_text: String,
    pub status_label: String,
}

pub fn to_regional_failover_variance_view(item: &RegionalFailoverVariance) -> RegionalFailoverVarianceView {
    RegionalFailoverVarianceView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        quantity_text: item.quantity.to_string(),
        created_at: item.created_at.to_string(),
        window_days: item.window_days.to_string(),
        margin_text: format!("{:.2}", item.margin),
        scope: item.scope.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        title: text::truncate(&item.title, 40),
        cluster_text: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_failover_variance_views(items: &[RegionalFailoverVariance]) -> Vec<RegionalFailoverVarianceView> {
    let mut views: Vec<RegionalFailoverVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_failover_variance_view(item));
    }
    views
}

pub fn sum_regional_failover_variance_view(view: &RegionalFailoverVarianceView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_created_at as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_failover_variance("code-144", "name-653");
        let view = to_regional_failover_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_failover_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
