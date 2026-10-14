use super::*;

use acme_kit::text;

/// How screens and exports show a batched quote assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteAssignmentView {
    pub code_text: String,
    pub name: String,
    pub ceiling: String,
    pub batch_size_text: String,
    pub status_label: String,
}

pub fn to_batched_quote_assignment_view(item: &BatchedQuoteAssignment) -> BatchedQuoteAssignmentView {
    BatchedQuoteAssignmentView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 40),
        ceiling: format!("{:.2}", item.ceiling),
        batch_size_text: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_quote_assignment_views(items: &[BatchedQuoteAssignment]) -> Vec<BatchedQuoteAssignmentView> {
    let mut views: Vec<BatchedQuoteAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_quote_assignment_view(item));
    }
    views
}

pub fn sum_batched_quote_assignment_view(view: &BatchedQuoteAssignmentView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_ceiling + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_quote_assignment("code-246", "name-157");
        let view = to_batched_quote_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_quote_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
