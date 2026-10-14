use super::*;

use acme_kit::text;

/// How screens and exports show a batched slo cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSloCycleView {
    pub code: String,
    pub name: String,
    pub reviewer_text: String,
    pub summary: String,
    pub label_text: String,
    pub source: String,
    pub host_text: String,
    pub ceiling_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_batched_slo_cycle_view(item: &BatchedSloCycle) -> BatchedSloCycleView {
    BatchedSloCycleView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        reviewer_text: text::truncate(&item.reviewer, 120),
        summary: item.summary.clone(),
        label_text: text::truncate(&item.label, 80),
        source: text::truncate(&item.source, 40),
        host_text: text::truncate(&item.host, 40),
        ceiling_text: format!("{:.2}", item.ceiling),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_slo_cycle_views(items: &[BatchedSloCycle]) -> Vec<BatchedSloCycleView> {
    let mut views: Vec<BatchedSloCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_slo_cycle_view(item));
    }
    views
}

pub fn sum_batched_slo_cycle_view(view: &BatchedSloCycleView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_slo_cycle("code-891", "name-933");
        let view = to_batched_slo_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_slo_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
