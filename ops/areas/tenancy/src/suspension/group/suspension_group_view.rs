use super::*;

use acme_kit::text;

/// How screens and exports show a suspension group.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionGroupView {
    pub code_text: String,
    pub title: String,
    pub maximum: String,
    pub quantity: String,
    pub priority: String,
    pub batch_size_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_suspension_group_view(item: &SuspensionGroup) -> SuspensionGroupView {
    SuspensionGroupView {
        code_text: text::truncate(&item.code, 120),
        title: item.title.clone(),
        maximum: format!("{:.2}", item.maximum),
        quantity: item.quantity.to_string(),
        priority: item.priority.to_string(),
        batch_size_text: item.batch_size.to_string(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_group_views(items: &[SuspensionGroup]) -> Vec<SuspensionGroupView> {
    let mut views: Vec<SuspensionGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_group_view(item));
    }
    views
}

pub fn sum_suspension_group_view(view: &SuspensionGroupView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_maximum + (parsed_quantity as f64) + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_group("code-873", "title-955");
        let view = to_suspension_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
