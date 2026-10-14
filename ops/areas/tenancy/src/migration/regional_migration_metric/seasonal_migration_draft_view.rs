use super::*;

/// How screens and exports show a seasonal migration draft.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMigrationDraftView {
    pub code: String,
    pub name_text: String,
    pub region: String,
    pub ttl_seconds_text: String,
    pub rate: String,
    pub checksum: String,
    pub host_text: String,
    pub port_text: String,
    pub scope: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_seasonal_migration_draft_view(item: &SeasonalMigrationDraft) -> SeasonalMigrationDraftView {
    SeasonalMigrationDraftView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        region: item.region.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        rate: format!("{:.2}", item.rate),
        checksum: item.checksum.clone(),
        host_text: item.host.clone(),
        port_text: item.port.to_string(),
        scope: item.scope.clone(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_migration_draft_views(items: &[SeasonalMigrationDraft]) -> Vec<SeasonalMigrationDraftView> {
    let mut views: Vec<SeasonalMigrationDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_migration_draft_view(item));
    }
    views
}

pub fn sum_seasonal_migration_draft_view(view: &SeasonalMigrationDraftView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_rate + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_migration_draft("code-81", "name-648");
        let view = to_seasonal_migration_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_migration_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
