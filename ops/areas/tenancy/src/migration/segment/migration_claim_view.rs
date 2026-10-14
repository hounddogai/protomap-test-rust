use super::*;

use acme_kit::text;

/// How screens and exports show a migration claim.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationClaimView {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub port: String,
    pub notes: String,
    pub threshold: String,
    pub attempts: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_migration_claim_view(item: &MigrationClaim) -> MigrationClaimView {
    MigrationClaimView {
        code: item.code.clone(),
        title: item.title.clone(),
        cluster: text::truncate(&item.cluster, 64),
        port: item.port.to_string(),
        notes: text::truncate(&item.notes, 64),
        threshold: format!("{:.2}", item.threshold),
        attempts: item.attempts.to_string(),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_claim_views(items: &[MigrationClaim]) -> Vec<MigrationClaimView> {
    let mut views: Vec<MigrationClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_claim_view(item));
    }
    views
}

pub fn sum_migration_claim_view(view: &MigrationClaimView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_port as f64 + parsed_threshold + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_claim("code-637", "title-181");
        let view = to_migration_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
