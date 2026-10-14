use super::*;

/// How screens and exports show a strict egress entry.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressEntryView {
    pub code: String,
    pub title_text: String,
    pub batch_size: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_strict_egress_entry_view(item: &StrictEgressEntry) -> StrictEgressEntryView {
    StrictEgressEntryView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        batch_size: item.batch_size.to_string(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_egress_entry_views(items: &[StrictEgressEntry]) -> Vec<StrictEgressEntryView> {
    let mut views: Vec<StrictEgressEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_egress_entry_view(item));
    }
    views
}

pub fn sum_strict_egress_entry_view(view: &StrictEgressEntryView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_egress_entry("code-383", "title-719");
        let view = to_strict_egress_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_egress_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
