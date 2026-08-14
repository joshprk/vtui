use core::{any::Any, cmp::Ordering, ops::Range};

use ratatui::{
    buffer::Buffer,
    style::{Color, Modifier, Style},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{
    arena::NodeId,
    events::{MouseDown, MouseDrag, MouseUp},
    input::MouseButton,
    layout::LogicalRect,
    transport::Event,
};

/// A stable key for selectable text emitted by one component.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextKey(u64);

impl TextKey {
    /// Creates a text key. Keys need only be unique within one component.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

/// The semantic separator inserted before a text draw when copying across draws.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextJoin {
    /// Concatenate directly with the preceding text draw.
    Inline,
    /// Insert one semantic newline.
    #[default]
    Line,
    /// Insert two semantic newlines.
    Paragraph,
}

impl TextJoin {
    fn separator(self) -> &'static str {
        match self {
            Self::Inline => "",
            Self::Line => "\n",
            Self::Paragraph => "\n\n",
        }
    }
}

/// Selection behavior for a [`Canvas::text_with`](crate::Canvas::text_with) draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextOptions {
    /// Optional identity which survives conditional draw ordering.
    pub key: Option<TextKey>,
    /// Separator used when copying across this draw boundary.
    pub join: TextJoin,
    /// Whether this draw contributes to semantic selection.
    pub selectable: bool,
}

impl Default for TextOptions {
    fn default() -> Self {
        Self {
            key: None,
            join: TextJoin::Line,
            selectable: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum RunKey {
    Implicit(u32),
    Explicit(TextKey),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct RunId {
    root: NodeId,
    node: NodeId,
    key: RunKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SelectionPoint {
    run: RunId,
    byte: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SelectionRange {
    anchor: SelectionPoint,
    focus: SelectionPoint,
}

#[derive(Clone, Debug)]
struct VisibleGrapheme {
    source: Range<usize>,
    screen: LogicalRect,
    visible: bool,
}

#[derive(Clone, Debug)]
struct TextRun {
    id: RunId,
    text: String,
    join: TextJoin,
    visible: Vec<VisibleGrapheme>,
}

#[derive(Clone, Copy, Debug)]
struct SelectionRoot {
    id: NodeId,
    area: LogicalRect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RunToken(usize);

/// One visible grapheme placement. Both Canvas painting and selection metadata consume this value.
#[derive(Clone, Debug)]
pub(crate) struct GraphemePlacement<'a> {
    pub(crate) symbol: &'a str,
    pub(crate) source: Range<usize>,
    pub(crate) screen: LogicalRect,
}

/// Removes graphemes which the current Canvas text renderer cannot display.
pub(crate) fn normalize_text(text: &str) -> String {
    UnicodeSegmentation::graphemes(text, true)
        .filter(|grapheme| !grapheme.contains(char::is_control))
        .collect()
}

/// Computes whole-grapheme placements in screen space.
///
/// This is the only horizontal clipping and width calculation used by semantic Canvas text.
pub(crate) fn layout_text(
    text: &str,
    origin_x: i32,
    row: i32,
    clip: LogicalRect,
    max_width: usize,
) -> Vec<GraphemePlacement<'_>> {
    if row < clip.top() || row >= clip.bottom() || clip.width <= 0 {
        return Vec::new();
    }

    let width_limit = i32::try_from(max_width).unwrap_or(i32::MAX);
    let logical_right = origin_x.saturating_add(width_limit);
    let visible_right = clip.right().min(logical_right);
    let mut column = origin_x;
    let mut placements = Vec::new();

    for (byte, grapheme) in UnicodeSegmentation::grapheme_indices(text, true) {
        let width = i32::try_from(grapheme.width()).unwrap_or(i32::MAX);
        if width == 0 {
            continue;
        }

        let end = column.saturating_add(width);
        if column >= clip.left() && end <= visible_right {
            placements.push(GraphemePlacement {
                symbol: grapheme,
                source: byte..byte + grapheme.len(),
                screen: LogicalRect::new(column, row, width, 1),
            });
        }

        column = end;
        if column >= visible_right {
            break;
        }
    }

    placements
}

#[derive(Default)]
pub(crate) struct SelectionController {
    frame: Vec<TextRun>,
    building: Vec<TextRun>,
    frame_roots: Vec<SelectionRoot>,
    building_roots: Vec<SelectionRoot>,
    selection: Option<SelectionRange>,
    captured_root: Option<NodeId>,
}

impl SelectionController {
    pub(crate) fn begin_frame(&mut self) {
        self.building.clear();
        self.building_roots.clear();
    }

    pub(crate) fn register_root(&mut self, id: NodeId, area: LogicalRect) {
        if area.width > 0 && area.height > 0 {
            self.building_roots.push(SelectionRoot { id, area });
        }
    }

    pub(crate) fn begin_text(
        &mut self,
        root: NodeId,
        node: NodeId,
        implicit_index: u32,
        text: String,
        options: TextOptions,
    ) -> Option<RunToken> {
        if !options.selectable {
            return None;
        }

        let key = options
            .key
            .map_or(RunKey::Implicit(implicit_index), RunKey::Explicit);
        let id = RunId { root, node, key };

        if self.building.iter().any(|run| run.id == id) {
            debug_assert!(false, "duplicate selectable text key in one component");
            return None;
        }

        let token = RunToken(self.building.len());
        self.building.push(TextRun {
            id,
            text,
            join: options.join,
            visible: Vec::new(),
        });
        Some(token)
    }

    pub(crate) fn record_visible(&mut self, token: RunToken, placement: &GraphemePlacement<'_>) {
        self.occlude(placement.screen);
        if let Some(run) = self.building.get_mut(token.0) {
            run.visible.push(VisibleGrapheme {
                source: placement.source.clone(),
                screen: placement.screen,
                visible: true,
            });
        }
    }

    pub(crate) fn occlude(&mut self, rect: LogicalRect) {
        if rect.width <= 0 || rect.height <= 0 {
            return;
        }

        for run in &mut self.building {
            for grapheme in &mut run.visible {
                if grapheme.visible && grapheme.screen.intersects(rect) {
                    grapheme.visible = false;
                }
            }
        }
    }

    pub(crate) fn occlude_all(&mut self, area: LogicalRect) {
        self.occlude(area);
    }

    pub(crate) fn finish_frame(&mut self, buffer: &mut Buffer) {
        self.reconcile_selection();
        self.apply_highlight(buffer);
        core::mem::swap(&mut self.frame, &mut self.building);
        self.building.clear();
        core::mem::swap(&mut self.frame_roots, &mut self.building_roots);
        self.building_roots.clear();
    }

    pub(crate) fn handle_event<E: Event>(&mut self, event: &E) {
        let event = event as &dyn Any;

        if let Some(event) = event.downcast_ref::<MouseDown>() {
            if event.button == MouseButton::Left {
                self.begin_gesture(event.x as i32, event.y as i32);
            }
            return;
        }

        if let Some(event) = event.downcast_ref::<MouseDrag>() {
            if event.button == MouseButton::Left {
                self.update_gesture(event.x as i32, event.y as i32);
            }
            return;
        }

        if let Some(event) = event.downcast_ref::<MouseUp>()
            && event.button == MouseButton::Left
        {
            self.update_gesture(event.x as i32, event.y as i32);
            self.captured_root = None;
        }
    }

    pub(crate) fn selected_text(&self) -> Option<String> {
        let selection = self.selection?;
        copy_range(&self.frame, selection)
    }

    fn begin_gesture(&mut self, x: i32, y: i32) {
        let exact = exact_hit(&self.frame, x, y, Ordering::Less);
        let nearest = root_at(&self.frame_roots, x, y)
            .and_then(|root| nearest_start_hit(&self.frame, root, x, y));
        let Some(point) = exact.or(nearest) else {
            self.selection = None;
            self.captured_root = None;
            return;
        };

        self.captured_root = Some(point.run.root);
        self.selection = Some(SelectionRange {
            anchor: point,
            focus: point,
        });
    }

    fn update_gesture(&mut self, x: i32, y: i32) {
        let (Some(root), Some(selection)) = (self.captured_root, self.selection) else {
            return;
        };
        let direction = pointer_direction(&self.frame, selection.focus, x, y);
        if let Some(point) = nearest_hit(&self.frame, root, x, y, direction) {
            self.selection = Some(SelectionRange {
                anchor: selection.anchor,
                focus: point,
            });
        }
    }

    fn reconcile_selection(&mut self) {
        let Some(mut selection) = self.selection else {
            return;
        };

        let Some(anchor_run) = self
            .building
            .iter()
            .find(|run| run.id == selection.anchor.run)
        else {
            self.selection = None;
            return;
        };
        selection.anchor.byte = clamp_grapheme_boundary(&anchor_run.text, selection.anchor.byte);

        let Some(focus_run) = self
            .building
            .iter()
            .find(|run| run.id == selection.focus.run)
        else {
            self.selection = None;
            return;
        };
        selection.focus.byte = clamp_grapheme_boundary(&focus_run.text, selection.focus.byte);
        self.selection = Some(selection);
    }

    fn apply_highlight(&self, buffer: &mut Buffer) {
        let Some(selection) = self.selection else {
            return;
        };
        let Some((start_index, start, end_index, end)) = normalize_range(&self.building, selection)
        else {
            return;
        };

        let selection_style = Style::default()
            .fg(Color::Black)
            .bg(Color::LightBlue)
            .add_modifier(Modifier::BOLD);

        for (index, run) in self.building.iter().enumerate() {
            if index < start_index || index > end_index {
                continue;
            }
            let lower = if index == start_index { start.byte } else { 0 };
            let upper = if index == end_index {
                end.byte
            } else {
                run.text.len()
            };

            for grapheme in run.visible.iter().filter(|grapheme| grapheme.visible) {
                if grapheme.source.start >= lower && grapheme.source.end <= upper {
                    patch_rect(buffer, grapheme.screen, selection_style);
                }
            }
        }
    }
}

fn root_at(roots: &[SelectionRoot], x: i32, y: i32) -> Option<NodeId> {
    roots
        .iter()
        .rev()
        .find(|root| contains(root.area, x, y))
        .map(|root| root.id)
}

fn patch_rect(buffer: &mut Buffer, rect: LogicalRect, style: Style) {
    let bounds = LogicalRect::from(buffer.area);
    let rect = rect.intersection(bounds);
    for y in rect.top()..rect.bottom() {
        for x in rect.left()..rect.right() {
            buffer[(x as u16, y as u16)].set_style(style);
        }
    }
}

fn exact_hit(runs: &[TextRun], x: i32, y: i32, direction: Ordering) -> Option<SelectionPoint> {
    runs.iter().rev().find_map(|run| {
        run.visible.iter().rev().find_map(|grapheme| {
            if grapheme.visible && contains(grapheme.screen, x, y) {
                Some(point_for_grapheme(run.id, grapheme, direction))
            } else {
                None
            }
        })
    })
}

fn nearest_hit(
    runs: &[TextRun],
    root: NodeId,
    x: i32,
    y: i32,
    direction: Ordering,
) -> Option<SelectionPoint> {
    if let Some(point) = exact_hit(runs, x, y, direction).filter(|point| point.run.root == root) {
        return Some(point);
    }

    closest_grapheme(runs, root, x, y)
        .map(|(id, grapheme)| point_for_grapheme(id, grapheme, direction))
}

fn nearest_start_hit(runs: &[TextRun], root: NodeId, x: i32, y: i32) -> Option<SelectionPoint> {
    closest_grapheme(runs, root, x, y).map(|(id, grapheme)| {
        let direction = (y, x).cmp(&(grapheme.screen.y, grapheme.screen.x));
        point_for_grapheme(id, grapheme, direction)
    })
}

fn closest_grapheme(
    runs: &[TextRun],
    root: NodeId,
    x: i32,
    y: i32,
) -> Option<(RunId, &VisibleGrapheme)> {
    runs.iter()
        .flat_map(|run| run.visible.iter().map(move |grapheme| (run.id, grapheme)))
        .filter(|(id, grapheme)| id.root == root && grapheme.visible)
        .min_by_key(|(_, grapheme)| distance_key(grapheme.screen, x, y))
}

fn distance_key(rect: LogicalRect, x: i32, y: i32) -> (i32, i32) {
    let vertical = if y < rect.top() {
        rect.top() - y
    } else if y >= rect.bottom() {
        y - rect.bottom() + 1
    } else {
        0
    };
    let horizontal = if x < rect.left() {
        rect.left() - x
    } else if x >= rect.right() {
        x - rect.right() + 1
    } else {
        0
    };
    (vertical, horizontal)
}

fn contains(rect: LogicalRect, x: i32, y: i32) -> bool {
    x >= rect.left() && x < rect.right() && y >= rect.top() && y < rect.bottom()
}

fn point_for_grapheme(
    run: RunId,
    grapheme: &VisibleGrapheme,
    direction: Ordering,
) -> SelectionPoint {
    let byte = if direction == Ordering::Greater {
        grapheme.source.end
    } else {
        grapheme.source.start
    };
    SelectionPoint { run, byte }
}

fn pointer_direction(runs: &[TextRun], focus: SelectionPoint, x: i32, y: i32) -> Ordering {
    let focus_position = runs
        .iter()
        .find(|run| run.id == focus.run)
        .and_then(|run| {
            run.visible.iter().find(|grapheme| {
                grapheme.source.contains(&focus.byte) || grapheme.source.end == focus.byte
            })
        })
        .map(|grapheme| (grapheme.screen.y, grapheme.screen.x));

    focus_position
        .map(|position| (y, x).cmp(&position))
        .unwrap_or(Ordering::Greater)
}

fn normalize_range(
    runs: &[TextRun],
    selection: SelectionRange,
) -> Option<(usize, SelectionPoint, usize, SelectionPoint)> {
    let anchor_index = runs.iter().position(|run| run.id == selection.anchor.run)?;
    let focus_index = runs.iter().position(|run| run.id == selection.focus.run)?;
    let forward = (anchor_index, selection.anchor.byte) <= (focus_index, selection.focus.byte);
    if forward {
        Some((anchor_index, selection.anchor, focus_index, selection.focus))
    } else {
        Some((focus_index, selection.focus, anchor_index, selection.anchor))
    }
}

fn copy_range(runs: &[TextRun], selection: SelectionRange) -> Option<String> {
    let (start_index, start, end_index, end) = normalize_range(runs, selection)?;
    if start_index == end_index && start.byte == end.byte {
        return Some(String::new());
    }

    let mut output = String::new();
    for (index, run) in runs
        .iter()
        .enumerate()
        .take(end_index + 1)
        .skip(start_index)
    {
        if index > start_index {
            output.push_str(run.join.separator());
        }
        let lower = if index == start_index { start.byte } else { 0 };
        let upper = if index == end_index {
            end.byte
        } else {
            run.text.len()
        };
        output.push_str(&run.text[lower..upper]);
    }
    Some(output)
}

fn clamp_grapheme_boundary(text: &str, requested: usize) -> usize {
    if requested >= text.len() {
        return text.len();
    }
    UnicodeSegmentation::grapheme_indices(text, true)
        .map(|(byte, _)| byte)
        .take_while(|byte| *byte <= requested)
        .last()
        .unwrap_or(0)
        .min(text.len())
}

#[cfg(test)]
mod tests {
    use ratatui::{buffer::Buffer, layout::Rect};
    use slotmap::SlotMap;

    use super::*;

    fn ids() -> (NodeId, NodeId) {
        let mut nodes = SlotMap::<NodeId, ()>::with_key();
        (nodes.insert(()), nodes.insert(()))
    }

    #[test]
    fn layout_uses_grapheme_boundaries_and_whole_wide_characters() {
        let placements = layout_text(
            "a好e\u{301}",
            0,
            0,
            LogicalRect::new(0, 0, 10, 1),
            usize::MAX,
        );
        assert_eq!(placements.len(), 3);
        assert_eq!(placements[1].screen, LogicalRect::new(1, 0, 2, 1));
        assert_eq!(placements[2].symbol, "e\u{301}");
    }

    #[test]
    fn clipping_never_emits_half_a_wide_grapheme() {
        let placements = layout_text("好x", 0, 0, LogicalRect::new(1, 0, 2, 1), usize::MAX);
        assert_eq!(placements.len(), 1);
        assert_eq!(placements[0].symbol, "x");
    }

    #[test]
    fn copying_uses_semantic_joins_not_visual_gaps() {
        let (root, node) = ids();
        let mut controller = SelectionController::default();
        controller.begin_frame();
        let first = controller
            .begin_text(root, node, 0, "hello".into(), TextOptions::default())
            .unwrap();
        let second = controller
            .begin_text(root, node, 1, "world".into(), TextOptions::default())
            .unwrap();
        controller.record_visible(
            first,
            &GraphemePlacement {
                symbol: "h",
                source: 0..1,
                screen: LogicalRect::new(0, 0, 1, 1),
            },
        );
        controller.record_visible(
            second,
            &GraphemePlacement {
                symbol: "d",
                source: 4..5,
                screen: LogicalRect::new(20, 8, 1, 1),
            },
        );
        controller.selection = Some(SelectionRange {
            anchor: SelectionPoint {
                run: controller.building[0].id,
                byte: 0,
            },
            focus: SelectionPoint {
                run: controller.building[1].id,
                byte: 5,
            },
        });
        let mut buffer = Buffer::empty(Rect::new(0, 0, 30, 10));
        controller.finish_frame(&mut buffer);
        assert_eq!(controller.selected_text().as_deref(), Some("hello\nworld"));
    }

    #[test]
    fn offscreen_runs_remain_in_copy_output() {
        let (root, node) = ids();
        let mut controller = SelectionController::default();
        controller.begin_frame();
        controller.begin_text(root, node, 0, "visible".into(), TextOptions::default());
        controller.begin_text(root, node, 1, "offscreen".into(), TextOptions::default());
        controller.begin_text(
            root,
            node,
            2,
            "visible again".into(),
            TextOptions::default(),
        );
        controller.selection = Some(SelectionRange {
            anchor: SelectionPoint {
                run: controller.building[0].id,
                byte: 0,
            },
            focus: SelectionPoint {
                run: controller.building[2].id,
                byte: 13,
            },
        });
        let mut buffer = Buffer::empty(Rect::new(0, 0, 1, 1));
        controller.finish_frame(&mut buffer);
        assert_eq!(
            controller.selected_text().as_deref(),
            Some("visible\noffscreen\nvisible again")
        );
    }

    #[test]
    fn active_drag_crosses_layout_whitespace() {
        let (root, node) = ids();
        let mut controller = SelectionController::default();
        controller.begin_frame();
        let first = controller
            .begin_text(root, node, 0, "first".into(), TextOptions::default())
            .unwrap();
        let second = controller
            .begin_text(root, node, 1, "second".into(), TextOptions::default())
            .unwrap();
        controller.record_visible(
            first,
            &GraphemePlacement {
                symbol: "f",
                source: 0..1,
                screen: LogicalRect::new(5, 2, 1, 1),
            },
        );
        controller.record_visible(
            second,
            &GraphemePlacement {
                symbol: "d",
                source: 5..6,
                screen: LogicalRect::new(5, 8, 1, 1),
            },
        );
        let mut buffer = Buffer::empty(Rect::new(0, 0, 20, 10));
        controller.finish_frame(&mut buffer);

        controller.begin_gesture(5, 2);
        controller.update_gesture(12, 6);

        assert_eq!(controller.selected_text().as_deref(), Some("first\nsecond"));
    }

    #[test]
    fn scrolling_changes_geometry_without_changing_semantic_range() {
        let (root, node) = ids();
        let mut controller = SelectionController::default();
        controller.begin_frame();
        let run = controller
            .begin_text(root, node, 0, "scrolling".into(), TextOptions::default())
            .unwrap();
        controller.record_visible(
            run,
            &GraphemePlacement {
                symbol: "s",
                source: 0..1,
                screen: LogicalRect::new(2, 4, 1, 1),
            },
        );
        controller.record_visible(
            run,
            &GraphemePlacement {
                symbol: "g",
                source: 8..9,
                screen: LogicalRect::new(10, 4, 1, 1),
            },
        );
        controller.selection = Some(SelectionRange {
            anchor: SelectionPoint {
                run: controller.building[0].id,
                byte: 0,
            },
            focus: SelectionPoint {
                run: controller.building[0].id,
                byte: 9,
            },
        });
        let mut buffer = Buffer::empty(Rect::new(0, 0, 20, 10));
        controller.finish_frame(&mut buffer);

        controller.begin_frame();
        controller.begin_text(root, node, 0, "scrolling".into(), TextOptions::default());
        controller.finish_frame(&mut buffer);

        assert_eq!(controller.selected_text().as_deref(), Some("scrolling"));
    }

    #[test]
    fn blank_space_outside_a_selection_root_cannot_start_selection() {
        let (root, node) = ids();
        let mut controller = SelectionController::default();
        controller.begin_frame();
        let run = controller
            .begin_text(root, node, 0, "text".into(), TextOptions::default())
            .unwrap();
        controller.record_visible(
            run,
            &GraphemePlacement {
                symbol: "t",
                source: 0..1,
                screen: LogicalRect::new(8, 3, 1, 1),
            },
        );
        let mut buffer = Buffer::empty(Rect::new(0, 0, 20, 5));
        controller.finish_frame(&mut buffer);

        controller.begin_gesture(1, 1);

        assert_eq!(controller.selected_text(), None);
    }

    #[test]
    fn blank_space_inside_a_selection_root_starts_at_nearest_text() {
        let (root, node) = ids();
        let mut controller = SelectionController::default();
        controller.begin_frame();
        controller.register_root(root, LogicalRect::new(0, 0, 20, 8));
        let first = controller
            .begin_text(root, node, 0, "first".into(), TextOptions::default())
            .unwrap();
        let second = controller
            .begin_text(root, node, 1, "second".into(), TextOptions::default())
            .unwrap();
        controller.record_visible(
            first,
            &GraphemePlacement {
                symbol: "f",
                source: 0..1,
                screen: LogicalRect::new(8, 2, 1, 1),
            },
        );
        controller.record_visible(
            second,
            &GraphemePlacement {
                symbol: "d",
                source: 5..6,
                screen: LogicalRect::new(8, 4, 1, 1),
            },
        );
        let mut buffer = Buffer::empty(Rect::new(0, 0, 20, 8));
        controller.finish_frame(&mut buffer);

        controller.begin_gesture(2, 2);
        controller.update_gesture(8, 4);

        assert_eq!(controller.selected_text().as_deref(), Some("first\nsecond"));
    }

    #[test]
    fn backward_selection_copies_in_document_order() {
        let (root, node) = ids();
        let first_id = RunId {
            root,
            node,
            key: RunKey::Implicit(0),
        };
        let second_id = RunId {
            root,
            node,
            key: RunKey::Implicit(1),
        };
        let runs = vec![
            TextRun {
                id: first_id,
                text: "first".into(),
                join: TextJoin::Line,
                visible: Vec::new(),
            },
            TextRun {
                id: second_id,
                text: "second".into(),
                join: TextJoin::Line,
                visible: Vec::new(),
            },
        ];
        let selection = SelectionRange {
            anchor: SelectionPoint {
                run: second_id,
                byte: 6,
            },
            focus: SelectionPoint {
                run: first_id,
                byte: 0,
            },
        };

        assert_eq!(
            copy_range(&runs, selection).as_deref(),
            Some("first\nsecond")
        );
    }

    #[test]
    fn captured_selection_does_not_cross_roots() {
        let (first_root, second_root) = ids();
        let mut nodes = SlotMap::<NodeId, ()>::with_key();
        let first_node = nodes.insert(());
        let second_node = nodes.insert(());
        let mut controller = SelectionController::default();
        controller.begin_frame();
        let first = controller
            .begin_text(
                first_root,
                first_node,
                0,
                "first".into(),
                TextOptions::default(),
            )
            .unwrap();
        let second = controller
            .begin_text(
                second_root,
                second_node,
                0,
                "second".into(),
                TextOptions::default(),
            )
            .unwrap();
        controller.record_visible(
            first,
            &GraphemePlacement {
                symbol: "f",
                source: 0..1,
                screen: LogicalRect::new(1, 1, 1, 1),
            },
        );
        controller.record_visible(
            second,
            &GraphemePlacement {
                symbol: "s",
                source: 0..1,
                screen: LogicalRect::new(1, 3, 1, 1),
            },
        );
        let mut buffer = Buffer::empty(Rect::new(0, 0, 10, 5));
        controller.finish_frame(&mut buffer);

        controller.begin_gesture(1, 1);
        controller.update_gesture(1, 3);

        assert_eq!(controller.selected_text().as_deref(), Some("f"));
    }

    #[test]
    fn occluded_text_is_not_highlighted() {
        let (root, node) = ids();
        let mut controller = SelectionController::default();
        controller.begin_frame();
        let run = controller
            .begin_text(root, node, 0, "好".into(), TextOptions::default())
            .unwrap();
        controller.record_visible(
            run,
            &GraphemePlacement {
                symbol: "好",
                source: 0..3,
                screen: LogicalRect::new(1, 0, 2, 1),
            },
        );
        controller.selection = Some(SelectionRange {
            anchor: SelectionPoint {
                run: controller.building[0].id,
                byte: 0,
            },
            focus: SelectionPoint {
                run: controller.building[0].id,
                byte: 3,
            },
        });
        controller.occlude(LogicalRect::new(1, 0, 2, 1));
        let mut buffer = Buffer::empty(Rect::new(0, 0, 4, 1));
        controller.finish_frame(&mut buffer);

        assert_eq!(buffer[(1, 0)].bg, Color::Reset);
        assert_eq!(buffer[(2, 0)].bg, Color::Reset);
    }

    #[test]
    fn wide_grapheme_highlight_styles_every_occupied_cell() {
        let (root, node) = ids();
        let mut controller = SelectionController::default();
        controller.begin_frame();
        let run = controller
            .begin_text(root, node, 0, "好".into(), TextOptions::default())
            .unwrap();
        controller.record_visible(
            run,
            &GraphemePlacement {
                symbol: "好",
                source: 0..3,
                screen: LogicalRect::new(1, 0, 2, 1),
            },
        );
        controller.selection = Some(SelectionRange {
            anchor: SelectionPoint {
                run: controller.building[0].id,
                byte: 0,
            },
            focus: SelectionPoint {
                run: controller.building[0].id,
                byte: 3,
            },
        });
        let mut buffer = Buffer::empty(Rect::new(0, 0, 4, 1));
        controller.finish_frame(&mut buffer);

        assert_eq!(buffer[(1, 0)].bg, Color::LightBlue);
        assert_eq!(buffer[(2, 0)].bg, Color::LightBlue);
    }
}
