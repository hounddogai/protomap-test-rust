use super::*;

/// How screens and exports show a peering entry.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringEntryView {
    pub code_text: String,
    pub title_text: String,
    pub visible_text: String,
    pub channel: String,
    pub discount_text: String,
    pub weight: String,
    pub label_text: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_peering_entry_view(item: &PeeringEntry) -> PeeringEntryView {
    PeeringEntryView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        visible_text: format!("{}", item.visible),
        channel: item.channel.clone(),
        discount_text: format!("{:.2}", item.discount),
        weight: format!("{:.2}", item.weight),
        label_text: item.label.clone(),
        scope: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_entry_views(items: &[PeeringEntry]) -> Vec<PeeringEntryView> {
    let mut views: Vec<PeeringEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_entry_view(item));
    }
    views
}

pub fn sum_peering_entry_view(view: &PeeringEntryView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_discount + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_entry("code-529", "title-368");
        let view = to_peering_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
