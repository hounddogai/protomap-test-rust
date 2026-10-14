use super::*;

use acme_kit::text;

/// How screens and exports show a egress score.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressScoreView {
    pub code_text: String,
    pub label: String,
    pub cipher: String,
    pub archived: String,
    pub floor_value: String,
    pub external_ref: String,
    pub name_text: String,
    pub issuer: String,
    pub locale: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_egress_score_view(item: &EgressScore) -> EgressScoreView {
    EgressScoreView {
        code_text: text::truncate(&item.code, 40),
        label: item.label.clone(),
        cipher: item.cipher.clone(),
        archived: format!("{}", item.archived),
        floor_value: format!("{:.2}", item.floor_value),
        external_ref: item.external_ref.clone(),
        name_text: item.name.clone(),
        issuer: item.issuer.clone(),
        locale: item.locale.clone(),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_score_views(items: &[EgressScore]) -> Vec<EgressScoreView> {
    let mut views: Vec<EgressScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_score_view(item));
    }
    views
}

pub fn sum_egress_score_view(view: &EgressScoreView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_score("code-504", "label-313");
        let view = to_egress_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
