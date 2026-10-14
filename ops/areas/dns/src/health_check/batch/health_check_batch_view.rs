use super::*;

use acme_kit::text;

/// How screens and exports show a health check batch.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBatchView {
    pub code: String,
    pub title: String,
    pub amount_text: String,
    pub external_ref_text: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_health_check_batch_view(item: &HealthCheckBatch) -> HealthCheckBatchView {
    HealthCheckBatchView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        amount_text: format!("{:.2}", item.amount),
        external_ref_text: item.external_ref.clone(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_batch_views(items: &[HealthCheckBatch]) -> Vec<HealthCheckBatchView> {
    let mut views: Vec<HealthCheckBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_batch_view(item));
    }
    views
}

pub fn sum_health_check_batch_view(view: &HealthCheckBatchView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_amount + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_batch("code-785", "title-21");
        let view = to_health_check_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
