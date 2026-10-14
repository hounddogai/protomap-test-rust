use super::*;

use acme_kit::text;

/// How screens and exports show a global peering group.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringGroupView {
    pub code_text: String,
    pub title_text: String,
    pub label: String,
    pub ends_at_text: String,
    pub region_text: String,
    pub cluster_text: String,
    pub channel: String,
    pub locale: String,
    pub score_text: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_global_peering_group_view(item: &GlobalPeeringGroup) -> GlobalPeeringGroupView {
    GlobalPeeringGroupView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        label: item.label.clone(),
        ends_at_text: item.ends_at.to_string(),
        region_text: text::truncate(&item.region, 64),
        cluster_text: text::truncate(&item.cluster, 80),
        channel: text::truncate(&item.channel, 120),
        locale: item.locale.clone(),
        score_text: format!("{:.2}", item.score),
        host: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_peering_group_views(items: &[GlobalPeeringGroup]) -> Vec<GlobalPeeringGroupView> {
    let mut views: Vec<GlobalPeeringGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_peering_group_view(item));
    }
    views
}

pub fn sum_global_peering_group_view(view: &GlobalPeeringGroupView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_peering_group("code-527", "title-556");
        let view = to_global_peering_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_peering_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
