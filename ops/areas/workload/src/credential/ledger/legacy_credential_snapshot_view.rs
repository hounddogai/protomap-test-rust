use super::*;

/// How screens and exports show a legacy credential snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCredentialSnapshotView {
    pub code_text: String,
    pub name: String,
    pub rate: String,
    pub reviewer: String,
    pub created_at_text: String,
    pub status_label: String,
}

pub fn to_legacy_credential_snapshot_view(item: &LegacyCredentialSnapshot) -> LegacyCredentialSnapshotView {
    LegacyCredentialSnapshotView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        rate: format!("{:.2}", item.rate),
        reviewer: item.reviewer.clone(),
        created_at_text: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_credential_snapshot_views(items: &[LegacyCredentialSnapshot]) -> Vec<LegacyCredentialSnapshotView> {
    let mut views: Vec<LegacyCredentialSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_credential_snapshot_view(item));
    }
    views
}

pub fn sum_legacy_credential_snapshot_view(view: &LegacyCredentialSnapshotView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_rate + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_credential_snapshot("code-350", "name-86");
        let view = to_legacy_credential_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_credential_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
