use super::*;

use acme_kit::text;

/// How screens and exports show a retention cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionCycleView {
    pub code: String,
    pub name: String,
    pub maximum: String,
    pub title: String,
    pub summary: String,
    pub limit: String,
    pub quantity: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_retention_cycle_view(item: &RetentionCycle) -> RetentionCycleView {
    RetentionCycleView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        maximum: format!("{:.2}", item.maximum),
        title: item.title.clone(),
        summary: item.summary.clone(),
        limit: item.limit.to_string(),
        quantity: item.quantity.to_string(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_cycle_views(items: &[RetentionCycle]) -> Vec<RetentionCycleView> {
    let mut views: Vec<RetentionCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_cycle_view(item));
    }
    views
}

pub fn sum_retention_cycle_view(view: &RetentionCycleView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_maximum + (parsed_limit as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_cycle("code-549", "name-330");
        let view = to_retention_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
