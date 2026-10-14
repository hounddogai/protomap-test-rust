use super::*;

use acme_kit::text;

/// How screens and exports show a usage checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageCheckpointView {
    pub code_text: String,
    pub name: String,
    pub discount: String,
    pub contact_email: String,
    pub country: String,
    pub fingerprint: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_usage_checkpoint_view(item: &UsageCheckpoint) -> UsageCheckpointView {
    UsageCheckpointView {
        code_text: text::truncate(&item.code, 64),
        name: item.name.clone(),
        discount: format!("{:.2}", item.discount),
        contact_email: text::truncate(&item.contact_email, 120),
        country: text::truncate(&item.country, 64),
        fingerprint: item.fingerprint.clone(),
        locale: text::truncate(&item.locale, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_checkpoint_views(items: &[UsageCheckpoint]) -> Vec<UsageCheckpointView> {
    let mut views: Vec<UsageCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_checkpoint_view(item));
    }
    views
}

pub fn sum_usage_checkpoint_view(view: &UsageCheckpointView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_checkpoint("code-157", "name-832");
        let view = to_usage_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
