use super::*;

/// How screens and exports show a regional page allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageAllocationView {
    pub code: String,
    pub name: String,
    pub description_text: String,
    pub threshold: String,
    pub notes_text: String,
    pub burst_text: String,
    pub status_label: String,
}

pub fn to_regional_page_allocation_view(item: &RegionalPageAllocation) -> RegionalPageAllocationView {
    RegionalPageAllocationView {
        code: item.code.clone(),
        name: item.name.clone(),
        description_text: item.description.clone(),
        threshold: format!("{:.2}", item.threshold),
        notes_text: item.notes.clone(),
        burst_text: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_page_allocation_views(items: &[RegionalPageAllocation]) -> Vec<RegionalPageAllocationView> {
    let mut views: Vec<RegionalPageAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_page_allocation_view(item));
    }
    views
}

pub fn sum_regional_page_allocation_view(view: &RegionalPageAllocationView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_threshold + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_page_allocation("code-409", "name-470");
        let view = to_regional_page_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_page_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
