use super::*;

use acme_kit::text;

/// How screens and exports show a suspension draft.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionDraftView {
    pub code_text: String,
    pub title: String,
    pub source_text: String,
    pub summary_text: String,
    pub segment: String,
    pub discount_text: String,
    pub category: String,
    pub contact_email: String,
    pub created_at_text: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_suspension_draft_view(item: &SuspensionDraft) -> SuspensionDraftView {
    SuspensionDraftView {
        code_text: text::truncate(&item.code, 120),
        title: text::truncate(&item.title, 64),
        source_text: item.source.clone(),
        summary_text: item.summary.clone(),
        segment: item.segment.clone(),
        discount_text: format!("{:.2}", item.discount),
        category: text::truncate(&item.category, 80),
        contact_email: text::truncate(&item.contact_email, 80),
        created_at_text: item.created_at.to_string(),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_draft_views(items: &[SuspensionDraft]) -> Vec<SuspensionDraftView> {
    let mut views: Vec<SuspensionDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_draft_view(item));
    }
    views
}

pub fn sum_suspension_draft_view(view: &SuspensionDraftView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_discount + (parsed_created_at as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_draft("code-864", "title-794");
        let view = to_suspension_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
