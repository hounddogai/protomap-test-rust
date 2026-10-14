use super::*;

use acme_kit::text;

/// How screens and exports show a overage digest.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageDigestView {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub contact_email_text: String,
    pub timezone: String,
    pub summary: String,
    pub key_id: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_overage_digest_view(item: &OverageDigest) -> OverageDigestView {
    OverageDigestView {
        code: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 40),
        fingerprint: item.fingerprint.clone(),
        contact_email_text: item.contact_email.clone(),
        timezone: item.timezone.clone(),
        summary: item.summary.clone(),
        key_id: item.key_id.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_digest_views(items: &[OverageDigest]) -> Vec<OverageDigestView> {
    let mut views: Vec<OverageDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_digest_view(item));
    }
    views
}

pub fn sum_overage_digest_view(view: &OverageDigestView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_digest("code-569", "name-464");
        let view = to_overage_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
