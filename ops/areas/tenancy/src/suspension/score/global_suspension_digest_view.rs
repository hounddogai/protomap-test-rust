use super::*;

use acme_kit::text;

/// How screens and exports show a global suspension digest.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSuspensionDigestView {
    pub code_text: String,
    pub title: String,
    pub contact_email: String,
    pub limit: String,
    pub owner_id_text: String,
    pub checksum_text: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_global_suspension_digest_view(item: &GlobalSuspensionDigest) -> GlobalSuspensionDigestView {
    GlobalSuspensionDigestView {
        code_text: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 120),
        contact_email: item.contact_email.clone(),
        limit: item.limit.to_string(),
        owner_id_text: text::truncate(&item.owner_id, 120),
        checksum_text: text::truncate(&item.checksum, 64),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_suspension_digest_views(items: &[GlobalSuspensionDigest]) -> Vec<GlobalSuspensionDigestView> {
    let mut views: Vec<GlobalSuspensionDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_suspension_digest_view(item));
    }
    views
}

pub fn sum_global_suspension_digest_view(view: &GlobalSuspensionDigestView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_suspension_digest("code-444", "title-437");
        let view = to_global_suspension_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_suspension_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
