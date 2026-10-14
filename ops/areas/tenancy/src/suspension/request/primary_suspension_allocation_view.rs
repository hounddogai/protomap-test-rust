use super::*;

use acme_kit::text;

/// How screens and exports show a primary suspension allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionAllocationView {
    pub code: String,
    pub title: String,
    pub duration_minutes_text: String,
    pub rate: String,
    pub weight: String,
    pub currency_text: String,
    pub batch_size: String,
    pub updated_at: String,
    pub notes: String,
    pub unit_text: String,
    pub status_label: String,
}

pub fn to_primary_suspension_allocation_view(item: &PrimarySuspensionAllocation) -> PrimarySuspensionAllocationView {
    PrimarySuspensionAllocationView {
        code: item.code.clone(),
        title: item.title.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        rate: format!("{:.2}", item.rate),
        weight: format!("{:.2}", item.weight),
        currency_text: item.currency.clone(),
        batch_size: item.batch_size.to_string(),
        updated_at: item.updated_at.to_string(),
        notes: item.notes.clone(),
        unit_text: text::truncate(&item.unit, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_suspension_allocation_views(
    items: &[PrimarySuspensionAllocation],
) -> Vec<PrimarySuspensionAllocationView> {
    let mut views: Vec<PrimarySuspensionAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_suspension_allocation_view(item));
    }
    views
}

pub fn sum_primary_suspension_allocation_view(view: &PrimarySuspensionAllocationView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_rate + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_suspension_allocation("code-313", "title-400");
        let view = to_primary_suspension_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_suspension_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
