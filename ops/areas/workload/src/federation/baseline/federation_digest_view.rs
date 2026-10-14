use super::*;

use acme_kit::text;

/// How screens and exports show a federation digest.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationDigestView {
    pub code: String,
    pub label: String,
    pub created_at: String,
    pub batch_size_text: String,
    pub starts_at_text: String,
    pub name: String,
    pub ratio: String,
    pub audience: String,
    pub host_text: String,
    pub phone_text: String,
    pub status_label: String,
}

pub fn to_federation_digest_view(item: &FederationDigest) -> FederationDigestView {
    FederationDigestView {
        code: item.code.clone(),
        label: item.label.clone(),
        created_at: item.created_at.to_string(),
        batch_size_text: item.batch_size.to_string(),
        starts_at_text: item.starts_at.to_string(),
        name: text::truncate(&item.name, 80),
        ratio: format!("{:.2}", item.ratio),
        audience: item.audience.clone(),
        host_text: item.host.clone(),
        phone_text: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_digest_views(items: &[FederationDigest]) -> Vec<FederationDigestView> {
    let mut views: Vec<FederationDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_digest_view(item));
    }
    views
}

pub fn sum_federation_digest_view(view: &FederationDigestView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_batch_size as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_digest("code-653", "label-437");
        let view = to_federation_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
