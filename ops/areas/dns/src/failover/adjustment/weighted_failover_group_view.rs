use super::*;

use acme_kit::text;

/// How screens and exports show a weighted failover group.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverGroupView {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub title: String,
    pub priority: String,
    pub source_text: String,
    pub minimum: String,
    pub amount: String,
    pub duration_minutes: String,
    pub rotation_days: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_weighted_failover_group_view(item: &WeightedFailoverGroup) -> WeightedFailoverGroupView {
    WeightedFailoverGroupView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        checksum: item.checksum.clone(),
        title: item.title.clone(),
        priority: item.priority.to_string(),
        source_text: item.source.clone(),
        minimum: format!("{:.2}", item.minimum),
        amount: format!("{:.2}", item.amount),
        duration_minutes: item.duration_minutes.to_string(),
        rotation_days: item.rotation_days.to_string(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_failover_group_views(items: &[WeightedFailoverGroup]) -> Vec<WeightedFailoverGroupView> {
    let mut views: Vec<WeightedFailoverGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_failover_group_view(item));
    }
    views
}

pub fn sum_weighted_failover_group_view(view: &WeightedFailoverGroupView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_priority as f64 + parsed_minimum + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_failover_group("code-1", "label-629");
        let view = to_weighted_failover_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_failover_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
