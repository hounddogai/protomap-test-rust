use super::*;

use acme_kit::text;

/// How screens and exports show a regional migration request.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationRequestView {
    pub code: String,
    pub name: String,
    pub port: String,
    pub issuer: String,
    pub cluster: String,
    pub duration_minutes: String,
    pub floor_value: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_regional_migration_request_view(item: &RegionalMigrationRequest) -> RegionalMigrationRequestView {
    RegionalMigrationRequestView {
        code: item.code.clone(),
        name: item.name.clone(),
        port: item.port.to_string(),
        issuer: item.issuer.clone(),
        cluster: text::truncate(&item.cluster, 80),
        duration_minutes: item.duration_minutes.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_migration_request_views(items: &[RegionalMigrationRequest]) -> Vec<RegionalMigrationRequestView> {
    let mut views: Vec<RegionalMigrationRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_migration_request_view(item));
    }
    views
}

pub fn sum_regional_migration_request_view(view: &RegionalMigrationRequestView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_port as f64 + (parsed_duration_minutes as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_migration_request("code-324", "name-836");
        let view = to_regional_migration_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_migration_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
