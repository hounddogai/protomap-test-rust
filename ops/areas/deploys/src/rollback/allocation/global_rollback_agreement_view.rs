use super::*;

use acme_kit::text;

/// How screens and exports show a global rollback agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRollbackAgreementView {
    pub code_text: String,
    pub name_text: String,
    pub pinned: String,
    pub ceiling: String,
    pub region: String,
    pub score: String,
    pub cipher: String,
    pub port: String,
    pub minimum_text: String,
    pub description_text: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_global_rollback_agreement_view(item: &GlobalRollbackAgreement) -> GlobalRollbackAgreementView {
    GlobalRollbackAgreementView {
        code_text: item.code.clone(),
        name_text: text::truncate(&item.name, 80),
        pinned: format!("{}", item.pinned),
        ceiling: format!("{:.2}", item.ceiling),
        region: item.region.clone(),
        score: format!("{:.2}", item.score),
        cipher: item.cipher.clone(),
        port: item.port.to_string(),
        minimum_text: format!("{:.2}", item.minimum),
        description_text: item.description.clone(),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_rollback_agreement_views(items: &[GlobalRollbackAgreement]) -> Vec<GlobalRollbackAgreementView> {
    let mut views: Vec<GlobalRollbackAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_rollback_agreement_view(item));
    }
    views
}

pub fn sum_global_rollback_agreement_view(view: &GlobalRollbackAgreementView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_ceiling + parsed_score + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_rollback_agreement("code-124", "name-783");
        let view = to_global_rollback_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_rollback_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
