use super::*;

use acme_kit::text;

/// How screens and exports show a migration request.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRequestView {
    pub code: String,
    pub name: String,
    pub archived: String,
    pub weight: String,
    pub cost: String,
    pub segment: String,
    pub quota_units: String,
    pub country: String,
    pub cluster_text: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_migration_request_view(item: &MigrationRequest) -> MigrationRequestView {
    MigrationRequestView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        archived: format!("{}", item.archived),
        weight: format!("{:.2}", item.weight),
        cost: format!("{:.2}", item.cost),
        segment: item.segment.clone(),
        quota_units: item.quota_units.to_string(),
        country: item.country.clone(),
        cluster_text: text::truncate(&item.cluster, 120),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_request_views(items: &[MigrationRequest]) -> Vec<MigrationRequestView> {
    let mut views: Vec<MigrationRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_request_view(item));
    }
    views
}

pub fn sum_migration_request_view(view: &MigrationRequestView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_weight + parsed_cost + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_request("code-804", "name-289");
        let view = to_migration_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
