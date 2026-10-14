use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive nonce profile.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceProfileView {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub locale: String,
    pub batch_size_text: String,
    pub status_label: String,
}

pub fn to_adaptive_nonce_profile_view(item: &AdaptiveNonceProfile) -> AdaptiveNonceProfileView {
    AdaptiveNonceProfileView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        checksum: item.checksum.clone(),
        locale: item.locale.clone(),
        batch_size_text: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_nonce_profile_views(items: &[AdaptiveNonceProfile]) -> Vec<AdaptiveNonceProfileView> {
    let mut views: Vec<AdaptiveNonceProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_nonce_profile_view(item));
    }
    views
}

pub fn sum_adaptive_nonce_profile_view(view: &AdaptiveNonceProfileView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_nonce_profile("code-911", "title-570");
        let view = to_adaptive_nonce_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_nonce_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
