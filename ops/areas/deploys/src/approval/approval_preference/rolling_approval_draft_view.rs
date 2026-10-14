use super::*;

use acme_kit::text;

/// How screens and exports show a rolling approval draft.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalDraftView {
    pub code_text: String,
    pub title: String,
    pub issuer: String,
    pub sequence: String,
    pub version: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_rolling_approval_draft_view(item: &RollingApprovalDraft) -> RollingApprovalDraftView {
    RollingApprovalDraftView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        issuer: text::truncate(&item.issuer, 120),
        sequence: item.sequence.to_string(),
        version: item.version.to_string(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_approval_draft_views(items: &[RollingApprovalDraft]) -> Vec<RollingApprovalDraftView> {
    let mut views: Vec<RollingApprovalDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_approval_draft_view(item));
    }
    views
}

pub fn sum_rolling_approval_draft_view(view: &RollingApprovalDraftView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_version as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_approval_draft("code-145", "title-563");
        let view = to_rolling_approval_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_approval_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
