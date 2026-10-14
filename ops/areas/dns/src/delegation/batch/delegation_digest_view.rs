use super::*;

use acme_kit::text;

/// How screens and exports show a delegation digest.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationDigestView {
    pub code: String,
    pub title: String,
    pub tenant_id_text: String,
    pub rate: String,
    pub visible_text: String,
    pub checksum: String,
    pub pinned: String,
    pub burst: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_delegation_digest_view(item: &DelegationDigest) -> DelegationDigestView {
    DelegationDigestView {
        code: item.code.clone(),
        title: item.title.clone(),
        tenant_id_text: item.tenant_id.clone(),
        rate: format!("{:.2}", item.rate),
        visible_text: format!("{}", item.visible),
        checksum: text::truncate(&item.checksum, 64),
        pinned: format!("{}", item.pinned),
        burst: item.burst.to_string(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_digest_views(items: &[DelegationDigest]) -> Vec<DelegationDigestView> {
    let mut views: Vec<DelegationDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_digest_view(item));
    }
    views
}

pub fn sum_delegation_digest_view(view: &DelegationDigestView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_rate + (parsed_burst as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_digest("code-737", "title-357");
        let view = to_delegation_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
