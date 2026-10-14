use super::*;

/// How screens and exports show a plan digest.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanDigestView {
    pub code_text: String,
    pub name_text: String,
    pub pinned_text: String,
    pub port: String,
    pub unit_text: String,
    pub region: String,
    pub description_text: String,
    pub burst: String,
    pub ttl_seconds: String,
    pub starts_at: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_plan_digest_view(item: &PlanDigest) -> PlanDigestView {
    PlanDigestView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        pinned_text: format!("{}", item.pinned),
        port: item.port.to_string(),
        unit_text: item.unit.clone(),
        region: item.region.clone(),
        description_text: item.description.clone(),
        burst: item.burst.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        starts_at: item.starts_at.to_string(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_digest_views(items: &[PlanDigest]) -> Vec<PlanDigestView> {
    let mut views: Vec<PlanDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_digest_view(item));
    }
    views
}

pub fn sum_plan_digest_view(view: &PlanDigestView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_port as f64 + (parsed_burst as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_digest("code-674", "name-584");
        let view = to_plan_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
