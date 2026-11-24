//! A small, render-agnostic tree that turns the resume data into something a
//! column browser can walk. Each node is one row in a column; `detail` is what
//! gets shown in the preview pane when the node is selected.

use crate::data::Portfolio;

/// A styled line in the preview pane. Keeping this independent of ratatui makes
/// the content easy to build and test without a terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetailLine {
    /// The node's name, e.g. "Software Engineer Intern @ Apera AI".
    Title(String),
    /// Supporting metadata, e.g. dates and location.
    Subtitle(String),
    /// A small section heading inside the detail pane.
    Heading(String),
    /// A wrapped paragraph.
    Text(String),
    /// A wrapped paragraph with a hanging bullet indent.
    Bullet(String),
    /// Vertical breathing room.
    Blank,
}

#[derive(Debug, Clone)]
pub struct Node {
    /// Text shown in the column row.
    pub label: String,
    /// Short secondary text shown under `label` in the column row.
    pub hint: String,
    /// Child columns revealed by pressing Right.
    pub children: Vec<Node>,
    /// Content shown in the preview pane when this node is selected.
    pub detail: Vec<DetailLine>,
}

impl Node {
    pub fn new(label: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            hint: hint.into(),
            children: Vec::new(),
            detail: Vec::new(),
        }
    }

    pub fn with_detail(mut self, detail: Vec<DetailLine>) -> Self {
        self.detail = detail;
        self
    }

    pub fn with_children(mut self, children: Vec<Node>) -> Self {
        self.children = children;
        self
    }
}

/// Build the full browse tree from embedded portfolio data.
pub fn build_tree(p: &Portfolio) -> Node {
    Node::new("Portfolio", &p.profile.name)
        .with_detail(vec![
            DetailLine::Title(p.profile.name.clone()),
            DetailLine::Subtitle(p.profile.tagline.clone()),
            DetailLine::Blank,
            DetailLine::Text(
                "Use ← / → to move between columns, ↑ / ↓ to move within a column, \
                 and q to quit."
                    .to_string(),
            ),
            DetailLine::Text(
                "Every column drills further into the resume; the rightmost pane \
                 shows the selected item's details."
                    .to_string(),
            ),
        ])
        .with_children(vec![
            about_node(p),
            experience_node(p),
            projects_node(p),
            skills_node(p),
            contact_node(p),
        ])
}

fn about_node(p: &Portfolio) -> Node {
    let profile = Node::new("Profile", "Name, tagline, location").with_detail(vec![
        DetailLine::Title(p.profile.name.clone()),
        DetailLine::Subtitle(p.profile.tagline.clone()),
        DetailLine::Blank,
        DetailLine::Text(format!("Location: {}", p.profile.location)),
        DetailLine::Text(format!("Phone: {}", p.profile.phone)),
        DetailLine::Text(format!("Email: {}", p.profile.email)),
    ]);

    let education = {
        let mut detail = vec![DetailLine::Heading("Education".to_string())];
        for e in &p.profile.education {
            detail.extend([
                DetailLine::Title(e.school.clone()),
                DetailLine::Subtitle(format!("{} · {}", e.degree, e.location)),
                DetailLine::Text(format!("{} – {}", e.start, e.end)),
            ]);
            if !e.detail.is_empty() {
                detail.push(DetailLine::Text(e.detail.clone()));
            }
            detail.push(DetailLine::Blank);
        }
        Node::new(
            "Education",
            format!("{} institution", p.profile.education.len()),
        )
        .with_detail(detail)
    };

    let links = {
        let mut detail = vec![DetailLine::Heading("Links".to_string())];
        for l in &p.profile.links {
            detail.push(DetailLine::Bullet(format!("{}: {}", l.label, l.url)));
        }
        Node::new("Links", format!("{} profiles", p.profile.links.len())).with_detail(detail)
    };

    Node::new("About", "Bio, education, links")
        .with_detail(vec![
            DetailLine::Title("About".to_string()),
            DetailLine::Subtitle(p.profile.tagline.clone()),
            DetailLine::Blank,
            DetailLine::Text(
                "Profile, education, and links. Drill in to read each part in the pane on the right."
                    .to_string(),
            ),
        ])
        .with_children(vec![profile, education, links])
}

fn experience_node(p: &Portfolio) -> Node {
    let roles = p
        .experience
        .iter()
        .map(|e| {
            let mut detail = vec![
                DetailLine::Title(format!("{} @ {}", e.role, e.company)),
                DetailLine::Subtitle(format!("{} – {} · {}", e.start, e.end, e.location)),
                DetailLine::Blank,
            ];
            for paragraph in e.body() {
                if e.is_prose() {
                    detail.push(DetailLine::Text(paragraph.clone()));
                } else {
                    detail.push(DetailLine::Bullet(paragraph.clone()));
                }
                detail.push(DetailLine::Blank);
            }
            if !e.tags.is_empty() {
                detail.push(DetailLine::Heading("Stack".to_string()));
                detail.push(DetailLine::Text(e.tags.join(" · ")));
            }
            Node::new(e.role.clone(), format!("{} · {}", e.company, e.start)).with_detail(detail)
        })
        .collect();

    Node::new("Experience", format!("{} roles", p.experience.len()))
        .with_detail(vec![
            DetailLine::Title("Experience".to_string()),
            DetailLine::Blank,
            DetailLine::Text(
                "Pick a role to read what I owned, the systems I worked on, and the \
             technologies involved."
                    .to_string(),
            ),
        ])
        .with_children(roles)
}

fn projects_node(p: &Portfolio) -> Node {
    let projects = p
        .projects
        .iter()
        .map(|pr| {
            let mut detail = vec![
                DetailLine::Title(pr.name.clone()),
                DetailLine::Subtitle(format!("{} · {}", pr.period, pr.stack.join(", "))),
                DetailLine::Blank,
                DetailLine::Text(pr.summary.clone()),
                DetailLine::Blank,
                DetailLine::Heading("Highlights".to_string()),
            ];
            for h in &pr.highlights {
                detail.push(DetailLine::Bullet(h.clone()));
            }
            if !pr.links.is_empty() {
                detail.push(DetailLine::Blank);
                detail.push(DetailLine::Heading("Links".to_string()));
                for l in &pr.links {
                    detail.push(DetailLine::Bullet(format!("{}: {}", l.label, l.url)));
                }
            }
            Node::new(pr.name.clone(), pr.period.clone()).with_detail(detail)
        })
        .collect();

    Node::new("Projects", format!("{} projects", p.projects.len()))
        .with_detail(vec![
            DetailLine::Title("Projects".to_string()),
            DetailLine::Blank,
            DetailLine::Text(
                "Side projects and shipped work. Each entry opens with a summary, \
                 highlights, and links."
                    .to_string(),
            ),
        ])
        .with_children(projects)
}

fn skills_node(p: &Portfolio) -> Node {
    let groups = p
        .skills
        .iter()
        .map(|g| {
            let mut detail = vec![DetailLine::Title(g.group.clone())];
            for item in &g.items {
                detail.push(DetailLine::Bullet(item.clone()));
            }
            Node::new(g.group.clone(), format!("{} skills", g.items.len())).with_detail(detail)
        })
        .collect();

    Node::new("Skills", format!("{} groups", p.skills.len()))
        .with_detail(vec![
            DetailLine::Title("Skills".to_string()),
            DetailLine::Blank,
            DetailLine::Text(
                "Grouped by languages, frameworks, and tools. Select a group to \
                 see its items."
                    .to_string(),
            ),
        ])
        .with_children(groups)
}

fn contact_node(p: &Portfolio) -> Node {
    let mut detail = vec![
        DetailLine::Title(p.profile.name.clone()),
        DetailLine::Text(format!("Email: {}", p.profile.email)),
        DetailLine::Text(format!("Phone: {}", p.profile.phone)),
        DetailLine::Blank,
        DetailLine::Heading("Online".to_string()),
    ];
    for l in &p.profile.links {
        detail.push(DetailLine::Bullet(format!("{}: {}", l.label, l.url)));
    }
    // Contact is a leaf: pressing Right jumps straight to the preview pane.
    Node::new("Contact", "Email, phone, profiles").with_detail(detail)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> Node {
        build_tree(&Portfolio::load())
    }

    #[test]
    fn root_has_all_resume_sections() {
        let root = tree();
        let labels: Vec<&str> = root.children.iter().map(|n| n.label.as_str()).collect();
        assert_eq!(
            labels,
            ["About", "Experience", "Projects", "Skills", "Contact"]
        );
    }

    #[test]
    fn every_leaf_has_readable_detail() {
        fn check(node: &Node) {
            if node.children.is_empty() {
                assert!(
                    node.detail.iter().any(|l| {
                        matches!(
                            l,
                            DetailLine::Title(_) | DetailLine::Text(_) | DetailLine::Bullet(_)
                        )
                    }),
                    "leaf '{}' has no readable detail",
                    node.label
                );
            }
            for child in &node.children {
                check(child);
            }
        }
        check(&tree());
    }

    #[test]
    fn experience_leaves_include_full_role_detail() {
        let root = tree();
        let experience = &root.children[1];
        let apera = &experience.children[0];
        let detail = apera
            .detail
            .iter()
            .filter_map(|l| match l {
                DetailLine::Text(s) | DetailLine::Bullet(s) => Some(s.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ");
        assert!(detail.contains("DAG-based job orchestration"));
        assert!(apera
            .detail
            .iter()
            .any(|l| matches!(l, DetailLine::Title(_))));
    }
}
