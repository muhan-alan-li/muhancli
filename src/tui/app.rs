//! Navigation state for the column browser. This module is deliberately free
//! of rendering code so the behavior can be unit tested without a terminal.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::tree::{build_tree, DetailLine, Node};
use crate::data::Portfolio;

#[derive(Debug)]
pub struct App {
    pub tree: Node,
    /// Selected index at each open list column, from left to right.
    pub cursor: Vec<usize>,
    /// Focused column index. `cursor.len()` means the preview pane.
    pub focus: usize,
    /// Vertical scroll offset of the preview pane.
    pub scroll: usize,
    pub quit: bool,
}

impl App {
    pub fn new(portfolio: &Portfolio) -> Self {
        Self {
            tree: build_tree(portfolio),
            cursor: vec![0],
            focus: 0,
            scroll: 0,
            quit: false,
        }
    }

    /// The node selected in list column `d`.
    pub fn node_at_depth(&self, d: usize) -> &Node {
        let mut node = &self.tree;
        for &idx in self.cursor.iter().take(d + 1) {
            node = &node.children[idx];
        }
        node
    }

    /// Rows shown in list column `d` (the children of the column to its left).
    pub fn column_nodes(&self, d: usize) -> &[Node] {
        if d == 0 {
            &self.tree.children
        } else {
            &self.node_at_depth(d - 1).children
        }
    }

    /// Number of list columns currently open.
    pub fn columns(&self) -> usize {
        self.cursor.len()
    }

    /// The node whose detail is shown in the preview pane.
    pub fn detail_node(&self) -> &Node {
        if self.cursor.is_empty() {
            return &self.tree;
        }
        let d = self.focus.min(self.cursor.len() - 1);
        self.node_at_depth(d)
    }

    pub fn detail(&self) -> &[DetailLine] {
        &self.detail_node().detail
    }

    /// Labels from the root to the deepest selected node, for the header.
    pub fn breadcrumb(&self) -> Vec<&str> {
        let mut labels = vec![self.tree.label.as_str()];
        for d in 0..self.cursor.len() {
            labels.push(self.node_at_depth(d).label.as_str());
        }
        labels
    }

    /// Drop columns whose selection no longer exists (e.g. after a tree
    /// rebuild) and keep `focus` in range.
    fn normalize(&mut self) {
        let mut node = &self.tree;
        let mut valid = 0;
        while valid < self.cursor.len() {
            let Some(child) = node.children.get(self.cursor[valid]) else {
                break;
            };
            node = child;
            valid += 1;
        }
        self.cursor.truncate(valid);
        self.focus = self.focus.min(self.cursor.len());
    }

    pub fn move_up(&mut self) {
        if self.focus < self.cursor.len() {
            let d = self.focus;
            let len = self.column_nodes(d).len();
            if len > 0 {
                self.cursor[d] = self.cursor[d].saturating_sub(1);
                self.cursor.truncate(d + 1);
            }
        } else {
            self.scroll = self.scroll.saturating_sub(1);
        }
        self.normalize();
    }

    pub fn move_down(&mut self) {
        if self.focus < self.cursor.len() {
            let d = self.focus;
            let len = self.column_nodes(d).len();
            if len > 0 {
                self.cursor[d] = (self.cursor[d] + 1).min(len - 1);
                self.cursor.truncate(d + 1);
            }
        } else {
            self.scroll = self.scroll.saturating_add(1);
        }
        self.normalize();
    }

    /// Right drills into the selected node when it has children, otherwise it
    /// moves focus to the preview pane.
    pub fn move_right(&mut self) {
        if self.focus >= self.cursor.len() {
            return;
        }
        let d = self.focus;
        let has_children = !self.node_at_depth(d).children.is_empty();
        if has_children {
            self.cursor.truncate(d + 1);
            self.cursor.push(0);
            self.focus = d + 1;
        } else {
            self.focus = self.cursor.len();
        }
        self.scroll = 0;
        self.normalize();
    }

    /// Left moves focus one column closer to the root without discarding the
    /// deeper columns.
    pub fn move_left(&mut self) {
        if self.focus > 0 {
            self.focus -= 1;
            self.scroll = 0;
        }
        self.normalize();
    }

    /// Close the deepest open column and focus its parent list column.
    pub fn collapse(&mut self) {
        if self.cursor.len() > 1 {
            self.cursor.pop();
            self.focus = self.focus.min(self.cursor.len().saturating_sub(1));
            self.scroll = 0;
        }
        self.normalize();
    }

    pub fn page_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(10);
    }

    pub fn page_down(&mut self) {
        self.scroll = self.scroll.saturating_add(10);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
            self.quit = true;
            return;
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::Left | KeyCode::Char('h') => self.move_left(),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => self.move_right(),
            KeyCode::Backspace => self.collapse(),
            KeyCode::PageUp => self.page_up(),
            KeyCode::PageDown => self.page_down(),
            KeyCode::Esc | KeyCode::Char('q') => self.quit = true,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        App::new(&Portfolio::load())
    }

    #[test]
    fn starts_on_the_first_category_with_about_in_the_pane() {
        let a = app();
        assert_eq!(a.cursor, [0]);
        assert_eq!(a.focus, 0);
        assert_eq!(a.columns(), 1);
        assert_eq!(a.detail_node().label, "About");
    }

    #[test]
    fn right_drills_into_children_then_focuses_the_pane() {
        let mut a = app();
        a.move_right();
        assert_eq!(a.cursor, [0, 0]);
        assert_eq!(a.focus, 1);
        assert_eq!(a.column_nodes(1)[0].label, "Profile");
        assert_eq!(a.detail_node().label, "Profile");

        a.move_right();
        assert_eq!(a.focus, 2);
        assert_eq!(a.detail_node().label, "Profile");
        a.move_right();
        assert_eq!(a.focus, 2, "stays in the pane at the deepest level");
    }

    #[test]
    fn left_walks_back_toward_the_root() {
        let mut a = app();
        a.move_right();
        a.move_right();
        assert_eq!(a.focus, 2);
        a.move_left();
        assert_eq!(a.focus, 1);
        a.move_left();
        assert_eq!(a.focus, 0);
        a.move_left();
        assert_eq!(a.focus, 0, "cannot go left of the first column");
        assert_eq!(a.cursor, [0, 0], "left keeps deeper columns open");
    }

    #[test]
    fn down_moves_selection_and_closes_stale_deeper_columns() {
        let mut a = app();
        a.move_right();
        a.move_right();
        assert_eq!(a.cursor, [0, 0]);
        a.move_left();
        a.move_down();
        assert_eq!(a.cursor, [0, 1]);
        assert_eq!(a.detail_node().label, "Education");
    }

    #[test]
    fn down_scrolls_the_pane_while_it_is_focused() {
        let mut a = app();
        a.move_right();
        a.move_right();
        assert_eq!(a.scroll, 0);
        a.move_down();
        assert_eq!(a.scroll, 1);
        a.move_up();
        assert_eq!(a.scroll, 0);
    }

    #[test]
    fn collapse_closes_the_deepest_column() {
        let mut a = app();
        a.move_right();
        assert_eq!(a.columns(), 2);
        a.collapse();
        assert_eq!(a.columns(), 1);
        assert_eq!(a.focus, 0);
    }

    #[test]
    fn keys_drive_the_same_actions() {
        let mut a = app();
        for key in ['l', 'l'] {
            a.handle_key(KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE));
        }
        assert_eq!(a.focus, 2);
        a.handle_key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE));
        assert_eq!(a.focus, 1);
        a.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        assert!(a.quit);
    }

    #[test]
    fn leaf_categories_jump_straight_to_the_pane() {
        let mut a = app();
        a.move_down(); // Experience
        a.move_down(); // Projects
        a.move_down(); // Skills
        a.move_down(); // Contact
        assert_eq!(a.detail_node().label, "Contact");
        a.move_right();
        assert_eq!(
            a.focus, 1,
            "contact has no children, so the pane is focused"
        );
        assert_eq!(a.detail_node().label, "Contact");
    }

    #[test]
    fn breadcrumb_tracks_the_selected_path() {
        let mut a = app();
        a.move_down(); // Experience
        a.move_right();
        assert_eq!(
            a.breadcrumb(),
            ["Portfolio", "Experience", "Software Engineer Intern"]
        );
    }
}
