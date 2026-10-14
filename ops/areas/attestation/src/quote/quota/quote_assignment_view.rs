use super::*;

use acme_kit::text;

/// How screens and exports show a quote assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAssignmentView {
    pub code: String,
    pub title: String,
    pub external_ref_text: String,
    pub ceiling: String,
    pub region_text: String,
    pub status_label: String,
}

pub fn to_quote_assignment_view(item: &QuoteAssignment) -> QuoteAssignmentView {
    QuoteAssignmentView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        external_ref_text: item.external_ref.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        region_text: text::truncate(&item.region, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_assignment_views(items: &[QuoteAssignment]) -> Vec<QuoteAssignmentView> {
    let mut views: Vec<QuoteAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_assignment_view(item));
    }
    views
}

pub fn sum_quote_assignment_view(view: &QuoteAssignmentView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_assignment("code-811", "title-462");
        let view = to_quote_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
