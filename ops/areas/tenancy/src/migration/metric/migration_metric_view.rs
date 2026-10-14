use super::*;

/// How screens and exports show a migration metric.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationMetricView {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub rotation_days: String,
    pub priority_text: String,
    pub region: String,
    pub minimum: String,
    pub currency: String,
    pub duration_minutes: String,
    pub updated_at: String,
    pub approved_text: String,
    pub status_label: String,
}

pub fn to_migration_metric_view(item: &MigrationMetric) -> MigrationMetricView {
    MigrationMetricView {
        code: item.code.clone(),
        name: item.name.clone(),
        phone: item.phone.clone(),
        rotation_days: item.rotation_days.to_string(),
        priority_text: item.priority.to_string(),
        region: item.region.clone(),
        minimum: format!("{:.2}", item.minimum),
        currency: item.currency.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        updated_at: item.updated_at.to_string(),
        approved_text: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_metric_views(items: &[MigrationMetric]) -> Vec<MigrationMetricView> {
    let mut views: Vec<MigrationMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_metric_view(item));
    }
    views
}

pub fn sum_migration_metric_view(view: &MigrationMetricView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_priority as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_metric("code-761", "name-574");
        let view = to_migration_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
