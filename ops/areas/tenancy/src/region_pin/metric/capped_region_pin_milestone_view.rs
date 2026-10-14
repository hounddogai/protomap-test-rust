use super::*;

use acme_kit::text;

/// How screens and exports show a capped region pin milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinMilestoneView {
    pub code_text: String,
    pub title: String,
    pub batch_size: String,
    pub amount: String,
    pub priority: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_capped_region_pin_milestone_view(item: &CappedRegionPinMilestone) -> CappedRegionPinMilestoneView {
    CappedRegionPinMilestoneView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 120),
        batch_size: item.batch_size.to_string(),
        amount: format!("{:.2}", item.amount),
        priority: item.priority.to_string(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_region_pin_milestone_views(items: &[CappedRegionPinMilestone]) -> Vec<CappedRegionPinMilestoneView> {
    let mut views: Vec<CappedRegionPinMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_region_pin_milestone_view(item));
    }
    views
}

pub fn sum_capped_region_pin_milestone_view(view: &CappedRegionPinMilestoneView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_amount + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_region_pin_milestone("code-273", "title-261");
        let view = to_capped_region_pin_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_region_pin_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
