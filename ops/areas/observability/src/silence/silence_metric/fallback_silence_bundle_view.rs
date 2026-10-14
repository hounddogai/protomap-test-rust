use super::*;

use acme_kit::text;

/// How screens and exports show a fallback silence bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSilenceBundleView {
    pub code: String,
    pub title_text: String,
    pub discount: String,
    pub namespace: String,
    pub country: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_fallback_silence_bundle_view(item: &FallbackSilenceBundle) -> FallbackSilenceBundleView {
    FallbackSilenceBundleView {
        code: text::truncate(&item.code, 120),
        title_text: text::truncate(&item.title, 120),
        discount: format!("{:.2}", item.discount),
        namespace: item.namespace.clone(),
        country: item.country.clone(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_silence_bundle_views(items: &[FallbackSilenceBundle]) -> Vec<FallbackSilenceBundleView> {
    let mut views: Vec<FallbackSilenceBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_silence_bundle_view(item));
    }
    views
}

pub fn sum_fallback_silence_bundle_view(view: &FallbackSilenceBundleView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_discount + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_silence_bundle("code-272", "title-872");
        let view = to_fallback_silence_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_silence_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
