use super::*;

/// How screens and exports show a global rollback draft.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRollbackDraftView {
    pub code: String,
    pub title_text: String,
    pub maximum: String,
    pub ceiling_text: String,
    pub summary_text: String,
    pub external_ref: String,
    pub rate: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_global_rollback_draft_view(item: &GlobalRollbackDraft) -> GlobalRollbackDraftView {
    GlobalRollbackDraftView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        maximum: format!("{:.2}", item.maximum),
        ceiling_text: format!("{:.2}", item.ceiling),
        summary_text: item.summary.clone(),
        external_ref: item.external_ref.clone(),
        rate: format!("{:.2}", item.rate),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_rollback_draft_views(items: &[GlobalRollbackDraft]) -> Vec<GlobalRollbackDraftView> {
    let mut views: Vec<GlobalRollbackDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_rollback_draft_view(item));
    }
    views
}

pub fn sum_global_rollback_draft_view(view: &GlobalRollbackDraftView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_maximum + parsed_ceiling + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_rollback_draft("code-680", "title-857");
        let view = to_global_rollback_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_rollback_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
