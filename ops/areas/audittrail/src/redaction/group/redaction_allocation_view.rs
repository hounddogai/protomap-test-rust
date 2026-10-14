use super::*;

use acme_kit::text;

/// How screens and exports show a redaction allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAllocationView {
    pub code: String,
    pub label: String,
    pub source_text: String,
    pub notes: String,
    pub summary: String,
    pub quantity_text: String,
    pub timezone: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_redaction_allocation_view(item: &RedactionAllocation) -> RedactionAllocationView {
    RedactionAllocationView {
        code: item.code.clone(),
        label: item.label.clone(),
        source_text: item.source.clone(),
        notes: item.notes.clone(),
        summary: text::truncate(&item.summary, 80),
        quantity_text: item.quantity.to_string(),
        timezone: item.timezone.clone(),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_allocation_views(items: &[RedactionAllocation]) -> Vec<RedactionAllocationView> {
    let mut views: Vec<RedactionAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_allocation_view(item));
    }
    views
}

pub fn sum_redaction_allocation_view(view: &RedactionAllocationView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_redaction_allocation("code-758", "label-161");
        let view = to_redaction_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_redaction_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
