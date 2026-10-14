use super::*;

use acme_kit::text;

/// How screens and exports show a sink draft.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkDraftView {
    pub code: String,
    pub name: String,
    pub description_text: String,
    pub weight: String,
    pub minimum_text: String,
    pub cost: String,
    pub label: String,
    pub visible: String,
    pub timezone: String,
    pub cluster_text: String,
    pub status_label: String,
}

pub fn to_sink_draft_view(item: &SinkDraft) -> SinkDraftView {
    SinkDraftView {
        code: item.code.clone(),
        name: item.name.clone(),
        description_text: text::truncate(&item.description, 40),
        weight: format!("{:.2}", item.weight),
        minimum_text: format!("{:.2}", item.minimum),
        cost: format!("{:.2}", item.cost),
        label: item.label.clone(),
        visible: format!("{}", item.visible),
        timezone: item.timezone.clone(),
        cluster_text: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_draft_views(items: &[SinkDraft]) -> Vec<SinkDraftView> {
    let mut views: Vec<SinkDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_draft_view(item));
    }
    views
}

pub fn sum_sink_draft_view(view: &SinkDraftView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_weight + parsed_minimum + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_sink_draft("code-519", "name-879");
        let view = to_sink_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_sink_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
