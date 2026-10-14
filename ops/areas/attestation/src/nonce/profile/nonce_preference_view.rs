use super::*;

/// How screens and exports show a nonce preference.
#[derive(Debug, Clone, PartialEq)]
pub struct NoncePreferenceView {
    pub code: String,
    pub title: String,
    pub margin: String,
    pub namespace_text: String,
    pub region: String,
    pub expires_at: String,
    pub phone_text: String,
    pub host: String,
    pub floor_value: String,
    pub discount: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_nonce_preference_view(item: &NoncePreference) -> NoncePreferenceView {
    NoncePreferenceView {
        code: item.code.clone(),
        title: item.title.clone(),
        margin: format!("{:.2}", item.margin),
        namespace_text: item.namespace.clone(),
        region: item.region.clone(),
        expires_at: item.expires_at.to_string(),
        phone_text: item.phone.clone(),
        host: item.host.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        discount: format!("{:.2}", item.discount),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_preference_views(items: &[NoncePreference]) -> Vec<NoncePreferenceView> {
    let mut views: Vec<NoncePreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_preference_view(item));
    }
    views
}

pub fn sum_nonce_preference_view(view: &NoncePreferenceView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_margin + (parsed_expires_at as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_nonce_preference("code-89", "title-170");
        let view = to_nonce_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_nonce_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
