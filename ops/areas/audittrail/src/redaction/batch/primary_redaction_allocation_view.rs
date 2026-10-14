use super::*;

use acme_kit::text;

/// How screens and exports show a primary redaction allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionAllocationView {
    pub code: String,
    pub title: String,
    pub ratio: String,
    pub quantity: String,
    pub description: String,
    pub grace_days: String,
    pub starts_at: String,
    pub port: String,
    pub window_days_text: String,
    pub unit: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_primary_redaction_allocation_view(item: &PrimaryRedactionAllocation) -> PrimaryRedactionAllocationView {
    PrimaryRedactionAllocationView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 64),
        ratio: format!("{:.2}", item.ratio),
        quantity: item.quantity.to_string(),
        description: item.description.clone(),
        grace_days: item.grace_days.to_string(),
        starts_at: item.starts_at.to_string(),
        port: item.port.to_string(),
        window_days_text: item.window_days.to_string(),
        unit: text::truncate(&item.unit, 64),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_redaction_allocation_views(
    items: &[PrimaryRedactionAllocation],
) -> Vec<PrimaryRedactionAllocationView> {
    let mut views: Vec<PrimaryRedactionAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_redaction_allocation_view(item));
    }
    views
}

pub fn sum_primary_redaction_allocation_view(view: &PrimaryRedactionAllocationView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_ratio + (parsed_quantity as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_redaction_allocation("code-830", "title-723");
        let view = to_primary_redaction_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_redaction_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
