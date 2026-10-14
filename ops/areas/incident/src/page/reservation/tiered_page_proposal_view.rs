use super::*;

use acme_kit::text;

/// How screens and exports show a tiered page proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageProposalView {
    pub code: String,
    pub label: String,
    pub rotation_days: String,
    pub region: String,
    pub floor_value: String,
    pub ceiling: String,
    pub minimum: String,
    pub cipher: String,
    pub visible: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_tiered_page_proposal_view(item: &TieredPageProposal) -> TieredPageProposalView {
    TieredPageProposalView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        rotation_days: item.rotation_days.to_string(),
        region: item.region.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        ceiling: format!("{:.2}", item.ceiling),
        minimum: format!("{:.2}", item.minimum),
        cipher: text::truncate(&item.cipher, 40),
        visible: format!("{}", item.visible),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_page_proposal_views(items: &[TieredPageProposal]) -> Vec<TieredPageProposalView> {
    let mut views: Vec<TieredPageProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_page_proposal_view(item));
    }
    views
}

pub fn sum_tiered_page_proposal_view(view: &TieredPageProposalView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_floor_value + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_page_proposal("code-995", "label-590");
        let view = to_tiered_page_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_page_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
