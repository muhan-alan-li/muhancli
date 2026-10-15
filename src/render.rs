use std::io::IsTerminal;

use crate::data::{Experience, Portfolio, Project, SkillGroup};

#[derive(Debug, Clone, Copy)]
pub struct OutputOpts {
    pub json: bool,
    pub plain: bool,
}

impl OutputOpts {
    pub fn colors_enabled(&self) -> bool {
        if self.plain || self.json {
            return false;
        }
        if std::env::var_os("NO_COLOR").is_some() {
            return false;
        }
        std::io::stdout().is_terminal()
    }
}

fn bold(s: &str, color: bool) -> String {
    if color {
        format!("\x1b[1m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

fn cyan(s: &str, color: bool) -> String {
    if color {
        format!("\x1b[36m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

fn dim(s: &str, color: bool) -> String {
    if color {
        format!("\x1b[2m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

fn bullet(point: &str, color: bool) -> String {
    if color {
        format!("  \x1b[32m•\x1b[0m {point}")
    } else {
        format!("  - {point}")
    }
}

pub fn render_about(p: &Portfolio, opts: OutputOpts) -> String {
    if opts.json {
        return serde_json::to_string_pretty(&p.profile).unwrap();
    }
    let c = opts.colors_enabled();
    let mut out = String::new();
    out.push_str(&format!("{}\n", bold(&p.profile.name, c)));
    out.push_str(&format!("{}\n", cyan(&p.profile.tagline, c)));
    out.push_str(&format!(
        "{}\n",
        dim(&format!("{} | {}", p.profile.location, p.profile.phone), c)
    ));
    out.push_str(&format!("{}\n", p.profile.email));
    for l in &p.profile.links {
        out.push_str(&format!("{}: {}\n", l.label, l.url));
    }
    out.push_str("\nEducation\n");
    for e in &p.profile.education {
        out.push_str(&format!(
            "  {} — {}\n    {} | {} ({} – {}) {}\n",
            bold(&e.school, c),
            e.degree,
            e.location,
            e.detail,
            e.start,
            e.end,
            ""
        ));
    }
    out.trim_end().to_string()
}

pub fn render_experience_list(p: &Portfolio, opts: OutputOpts) -> String {
    if opts.json {
        return serde_json::to_string_pretty(&p.experience).unwrap();
    }
    let c = opts.colors_enabled();
    let mut out = String::new();
    for e in &p.experience {
        out.push_str(&format!(
            "{:<10}  {} @ {}\n           {} — {} | {}\n",
            format!("[{}]", e.id),
            bold(&e.role, c),
            e.company,
            e.start,
            e.end,
            e.location
        ));
    }
    out.push_str(&dim("\nRun `muhan experience <id>` for detail.", c));
    out.trim_end().to_string()
}

pub fn render_experience_show(e: &Experience, opts: OutputOpts) -> String {
    if opts.json {
        return serde_json::to_string_pretty(e).unwrap();
    }
    let c = opts.colors_enabled();
    let mut out = String::new();
    out.push_str(&format!("{} @ {}\n", bold(&e.role, c), e.company));
    out.push_str(&format!("{} — {} | {}\n", e.start, e.end, e.location));
    if !e.tags.is_empty() {
        out.push_str(&format!("{}\n", dim(&e.tags.join(" · "), c)));
    }
    out.push('\n');
    for b in &e.bullets {
        out.push_str(&format!("{}\n", bullet(b, c)));
    }
    out.trim_end().to_string()
}

pub fn render_projects_list(projects: &[&Project], tag: Option<&str>, opts: OutputOpts) -> String {
    if opts.json {
        let owned: Vec<&Project> = projects.to_vec();
        return serde_json::to_string_pretty(&owned).unwrap();
    }
    let c = opts.colors_enabled();
    let mut out = String::new();
    if let Some(t) = tag {
        out.push_str(&format!("{}\n\n", dim(&format!("filter: {t}"), c)));
    }
    for p in projects {
        out.push_str(&format!(
            "{:<16}  {}\n               {} | {}\n",
            format!("[{}]", p.slug),
            bold(&p.name, c),
            p.period,
            p.stack.join(", ")
        ));
    }
    if projects.is_empty() {
        out.push_str("No projects matched.\n");
    } else {
        out.push_str(&dim("\nRun `muhan projects <slug>` for detail.", c));
    }
    out.trim_end().to_string()
}

pub fn render_project_show(p: &Project, opts: OutputOpts) -> String {
    if opts.json {
        return serde_json::to_string_pretty(p).unwrap();
    }
    let c = opts.colors_enabled();
    let mut out = String::new();
    out.push_str(&format!("{}\n", bold(&p.name, c)));
    out.push_str(&format!(
        "{}\n",
        dim(&format!("{} | {}", p.period, p.stack.join(", ")), c)
    ));
    out.push_str(&format!("{}\n\n", p.summary));
    for h in &p.highlights {
        out.push_str(&format!("{}\n", bullet(h, c)));
    }
    for l in &p.links {
        out.push_str(&format!("\n{}: {}", l.label, l.url));
    }
    out.trim_end().to_string()
}

pub fn render_skills(groups: &[&SkillGroup], opts: OutputOpts) -> String {
    if opts.json {
        let deref: Vec<&SkillGroup> = groups.to_vec();
        return serde_json::to_string_pretty(&deref).unwrap();
    }
    let c = opts.colors_enabled();
    let mut out = String::new();
    for g in groups {
        out.push_str(&format!("{}\n", bold(&g.group, c)));
        out.push_str(&format!("  {}\n\n", g.items.join(" · ")));
    }
    out.trim_end().to_string()
}

pub fn render_contact(p: &Portfolio, opts: OutputOpts) -> String {
    if opts.json {
        let v = serde_json::json!({
            "name": p.profile.name,
            "email": p.profile.email,
            "phone": p.profile.phone,
            "links": p.profile.links,
        });
        return serde_json::to_string_pretty(&v).unwrap();
    }
    let c = opts.colors_enabled();
    let mut out = String::new();
    out.push_str(&format!("{}\n", bold(&p.profile.name, c)));
    out.push_str(&format!("Email: {}\n", p.profile.email));
    out.push_str(&format!("Phone: {}\n", p.profile.phone));
    for l in &p.profile.links {
        out.push_str(&format!("{}: {}\n", l.label, l.url));
    }
    out.trim_end().to_string()
}

pub fn render_version() -> String {
    format!(
        "muhan {} ({} {})",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_SHA"),
        env!("BUILD_DATE")
    )
}
