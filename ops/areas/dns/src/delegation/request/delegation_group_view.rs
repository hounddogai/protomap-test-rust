use super::*;

use acme_kit::text;

/// How screens and exports show a delegation group.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationGroupView {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub phone: String,
    pub principal_text: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_delegation_group_view(item: &DelegationGroup) -> DelegationGroupView {
    DelegationGroupView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        summary: item.summary.clone(),
        phone: item.phone.clone(),
        principal_text: item.principal.clone(),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_group_views(items: &[DelegationGroup]) -> Vec<DelegationGroupView> {
    let mut views: Vec<DelegationGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_group_view(item));
    }
    views
}

pub fn sum_delegation_group_view(view: &DelegationGroupView) -> Result<f64, String> {
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
        let item = new_delegation_group("code-365", "label-623");
        let view = to_delegation_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
