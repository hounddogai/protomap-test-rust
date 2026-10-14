use super::*;

use acme_kit::text;

/// How screens and exports show a health check record.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRecordView {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub cost: String,
    pub margin: String,
    pub burst_text: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_health_check_record_view(item: &HealthCheckRecord) -> HealthCheckRecordView {
    HealthCheckRecordView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        segment: text::truncate(&item.segment, 120),
        cost: format!("{:.2}", item.cost),
        margin: format!("{:.2}", item.margin),
        burst_text: item.burst.to_string(),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_record_views(items: &[HealthCheckRecord]) -> Vec<HealthCheckRecordView> {
    let mut views: Vec<HealthCheckRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_record_view(item));
    }
    views
}

pub fn sum_health_check_record_view(view: &HealthCheckRecordView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_cost + parsed_margin + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_record("code-774", "name-322");
        let view = to_health_check_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
