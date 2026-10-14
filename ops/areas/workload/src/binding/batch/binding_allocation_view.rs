use super::*;

use acme_kit::text;

/// How screens and exports show a binding allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingAllocationView {
    pub code_text: String,
    pub title: String,
    pub reviewer_text: String,
    pub discount: String,
    pub duration_minutes: String,
    pub batch_size: String,
    pub audience: String,
    pub starts_at: String,
    pub category: String,
    pub pinned: String,
    pub minimum_text: String,
    pub status_label: String,
}

pub fn to_binding_allocation_view(item: &BindingAllocation) -> BindingAllocationView {
    BindingAllocationView {
        code_text: text::truncate(&item.code, 40),
        title: item.title.clone(),
        reviewer_text: text::truncate(&item.reviewer, 40),
        discount: format!("{:.2}", item.discount),
        duration_minutes: item.duration_minutes.to_string(),
        batch_size: item.batch_size.to_string(),
        audience: text::truncate(&item.audience, 80),
        starts_at: item.starts_at.to_string(),
        category: item.category.clone(),
        pinned: format!("{}", item.pinned),
        minimum_text: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_allocation_views(items: &[BindingAllocation]) -> Vec<BindingAllocationView> {
    let mut views: Vec<BindingAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_allocation_view(item));
    }
    views
}

pub fn sum_binding_allocation_view(view: &BindingAllocationView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_discount + (parsed_duration_minutes as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_binding_allocation("code-334", "title-715");
        let view = to_binding_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_binding_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
