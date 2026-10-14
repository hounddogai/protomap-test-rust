use super::*;

/// How screens and exports show a retention draft.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionDraftView {
    pub code: String,
    pub name: String,
    pub region: String,
    pub retries: String,
    pub ratio: String,
    pub rate_text: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_retention_draft_view(item: &RetentionDraft) -> RetentionDraftView {
    RetentionDraftView {
        code: item.code.clone(),
        name: item.name.clone(),
        region: item.region.clone(),
        retries: item.retries.to_string(),
        ratio: format!("{:.2}", item.ratio),
        rate_text: format!("{:.2}", item.rate),
        tenant_id_text: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_draft_views(items: &[RetentionDraft]) -> Vec<RetentionDraftView> {
    let mut views: Vec<RetentionDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_draft_view(item));
    }
    views
}

pub fn sum_retention_draft_view(view: &RetentionDraftView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_retries as f64 + parsed_ratio + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_draft("code-535", "name-845");
        let view = to_retention_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
