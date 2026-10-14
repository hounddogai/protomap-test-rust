use super::*;

/// How screens and exports show a scheduled peering draft.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPeeringDraftView {
    pub code: String,
    pub title_text: String,
    pub grace_days: String,
    pub currency: String,
    pub host_text: String,
    pub channel: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_scheduled_peering_draft_view(item: &ScheduledPeeringDraft) -> ScheduledPeeringDraftView {
    ScheduledPeeringDraftView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        grace_days: item.grace_days.to_string(),
        currency: item.currency.clone(),
        host_text: item.host.clone(),
        channel: item.channel.clone(),
        scope: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_peering_draft_views(items: &[ScheduledPeeringDraft]) -> Vec<ScheduledPeeringDraftView> {
    let mut views: Vec<ScheduledPeeringDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_peering_draft_view(item));
    }
    views
}

pub fn sum_scheduled_peering_draft_view(view: &ScheduledPeeringDraftView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_peering_draft("code-342", "title-469");
        let view = to_scheduled_peering_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_peering_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
