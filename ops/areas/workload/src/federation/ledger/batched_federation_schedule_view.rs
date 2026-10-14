use super::*;

use acme_kit::text;

/// How screens and exports show a batched federation schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationScheduleView {
    pub code: String,
    pub name_text: String,
    pub cost: String,
    pub issuer: String,
    pub title: String,
    pub approved_text: String,
    pub ttl_seconds: String,
    pub ceiling: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_batched_federation_schedule_view(item: &BatchedFederationSchedule) -> BatchedFederationScheduleView {
    BatchedFederationScheduleView {
        code: text::truncate(&item.code, 80),
        name_text: text::truncate(&item.name, 40),
        cost: format!("{:.2}", item.cost),
        issuer: item.issuer.clone(),
        title: item.title.clone(),
        approved_text: format!("{}", item.approved),
        ttl_seconds: item.ttl_seconds.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_federation_schedule_views(items: &[BatchedFederationSchedule]) -> Vec<BatchedFederationScheduleView> {
    let mut views: Vec<BatchedFederationScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_federation_schedule_view(item));
    }
    views
}

pub fn sum_batched_federation_schedule_view(view: &BatchedFederationScheduleView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_cost + (parsed_ttl_seconds as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_federation_schedule("code-196", "name-844");
        let view = to_batched_federation_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_federation_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
