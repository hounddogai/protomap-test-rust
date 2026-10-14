use super::*;

use acme_kit::text;

/// How screens and exports show a default freeze profile.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFreezeProfileView {
    pub code: String,
    pub name: String,
    pub label_text: String,
    pub external_ref: String,
    pub score: String,
    pub retries: String,
    pub contact_email_text: String,
    pub fingerprint_text: String,
    pub ends_at_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_default_freeze_profile_view(item: &DefaultFreezeProfile) -> DefaultFreezeProfileView {
    DefaultFreezeProfileView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        label_text: text::truncate(&item.label, 120),
        external_ref: item.external_ref.clone(),
        score: format!("{:.2}", item.score),
        retries: item.retries.to_string(),
        contact_email_text: item.contact_email.clone(),
        fingerprint_text: text::truncate(&item.fingerprint, 80),
        ends_at_text: item.ends_at.to_string(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_freeze_profile_views(items: &[DefaultFreezeProfile]) -> Vec<DefaultFreezeProfileView> {
    let mut views: Vec<DefaultFreezeProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_freeze_profile_view(item));
    }
    views
}

pub fn sum_default_freeze_profile_view(view: &DefaultFreezeProfileView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_score + (parsed_retries as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_freeze_profile("code-651", "name-124");
        let view = to_default_freeze_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_freeze_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
