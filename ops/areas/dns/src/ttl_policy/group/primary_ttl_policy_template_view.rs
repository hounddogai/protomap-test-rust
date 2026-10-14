use super::*;

/// How screens and exports show a primary ttl policy template.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyTemplateView {
    pub code_text: String,
    pub title: String,
    pub created_at: String,
    pub port_text: String,
    pub limit_text: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_primary_ttl_policy_template_view(item: &PrimaryTtlPolicyTemplate) -> PrimaryTtlPolicyTemplateView {
    PrimaryTtlPolicyTemplateView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        created_at: item.created_at.to_string(),
        port_text: item.port.to_string(),
        limit_text: item.limit.to_string(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_ttl_policy_template_views(items: &[PrimaryTtlPolicyTemplate]) -> Vec<PrimaryTtlPolicyTemplateView> {
    let mut views: Vec<PrimaryTtlPolicyTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_ttl_policy_template_view(item));
    }
    views
}

pub fn sum_primary_ttl_policy_template_view(view: &PrimaryTtlPolicyTemplateView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_port as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_ttl_policy_template("code-697", "title-348");
        let view = to_primary_ttl_policy_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_ttl_policy_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
