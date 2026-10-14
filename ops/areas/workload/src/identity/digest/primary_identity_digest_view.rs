use super::*;

/// How screens and exports show a primary identity digest.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityDigestView {
    pub code_text: String,
    pub title: String,
    pub duration_minutes: String,
    pub updated_at: String,
    pub ends_at: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_primary_identity_digest_view(item: &PrimaryIdentityDigest) -> PrimaryIdentityDigestView {
    PrimaryIdentityDigestView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        updated_at: item.updated_at.to_string(),
        ends_at: item.ends_at.to_string(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_identity_digest_views(items: &[PrimaryIdentityDigest]) -> Vec<PrimaryIdentityDigestView> {
    let mut views: Vec<PrimaryIdentityDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_identity_digest_view(item));
    }
    views
}

pub fn sum_primary_identity_digest_view(view: &PrimaryIdentityDigestView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_updated_at as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_identity_digest("code-551", "title-751");
        let view = to_primary_identity_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_identity_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
