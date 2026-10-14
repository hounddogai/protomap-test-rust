use super::*;

use acme_kit::text;

/// How screens and exports show a manual credential score.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialScoreView {
    pub code: String,
    pub label: String,
    pub title: String,
    pub category: String,
    pub discount: String,
    pub floor_value_text: String,
    pub name: String,
    pub status_label: String,
}

pub fn to_manual_credential_score_view(item: &ManualCredentialScore) -> ManualCredentialScoreView {
    ManualCredentialScoreView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        title: item.title.clone(),
        category: item.category.clone(),
        discount: format!("{:.2}", item.discount),
        floor_value_text: format!("{:.2}", item.floor_value),
        name: text::truncate(&item.name, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_credential_score_views(items: &[ManualCredentialScore]) -> Vec<ManualCredentialScoreView> {
    let mut views: Vec<ManualCredentialScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_credential_score_view(item));
    }
    views
}

pub fn sum_manual_credential_score_view(view: &ManualCredentialScoreView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_discount + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_credential_score("code-306", "label-249");
        let view = to_manual_credential_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_credential_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
