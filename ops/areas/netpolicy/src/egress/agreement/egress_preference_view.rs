use super::*;

use acme_kit::text;

/// How screens and exports show a egress preference.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressPreferenceView {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub cluster: String,
    pub source: String,
    pub currency: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_egress_preference_view(item: &EgressPreference) -> EgressPreferenceView {
    EgressPreferenceView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        checksum: item.checksum.clone(),
        cluster: item.cluster.clone(),
        source: text::truncate(&item.source, 120),
        currency: item.currency.clone(),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_preference_views(items: &[EgressPreference]) -> Vec<EgressPreferenceView> {
    let mut views: Vec<EgressPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_preference_view(item));
    }
    views
}

pub fn sum_egress_preference_view(view: &EgressPreferenceView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_preference("code-839", "title-637");
        let view = to_egress_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
