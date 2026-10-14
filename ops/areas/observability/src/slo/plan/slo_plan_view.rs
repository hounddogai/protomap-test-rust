use super::*;

use acme_kit::text;

/// How screens and exports show a slo plan.
#[derive(Debug, Clone, PartialEq)]
pub struct SloPlanView {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub sequence: String,
    pub ends_at: String,
    pub ceiling: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_slo_plan_view(item: &SloPlan) -> SloPlanView {
    SloPlanView {
        code: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 120),
        channel: text::truncate(&item.channel, 40),
        sequence: item.sequence.to_string(),
        ends_at: item.ends_at.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        principal: text::truncate(&item.principal, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_plan_views(items: &[SloPlan]) -> Vec<SloPlanView> {
    let mut views: Vec<SloPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_plan_view(item));
    }
    views
}

pub fn sum_slo_plan_view(view: &SloPlanView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_ends_at as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_plan("code-320", "label-767");
        let view = to_slo_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
