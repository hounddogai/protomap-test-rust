use super::*;

use acme_kit::text;

/// How screens and exports show a silence allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAllocationView {
    pub code_text: String,
    pub name: String,
    pub description_text: String,
    pub owner_id: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_silence_allocation_view(item: &SilenceAllocation) -> SilenceAllocationView {
    SilenceAllocationView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 40),
        description_text: item.description.clone(),
        owner_id: text::truncate(&item.owner_id, 40),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_allocation_views(items: &[SilenceAllocation]) -> Vec<SilenceAllocationView> {
    let mut views: Vec<SilenceAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_allocation_view(item));
    }
    views
}

pub fn sum_silence_allocation_view(view: &SilenceAllocationView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_allocation("code-608", "name-810");
        let view = to_silence_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
