# HTML-style text selection for vtui

Status: design baseline. An initial implementation now lives in `src/selection.rs`; staged and future sections remain the target architecture where they exceed that implementation.  
Source baseline: `main` at `254ac78` (vtui 0.6.0), inspected 2026-08-14.

## 1. Executive decision

vtui should implement selection as a semantic range over text emissions, not as a rectangle in the Ratatui buffer and not as a property of focusable component rectangles.

The recommended design is a hybrid:

1. `Canvas::text` emits semantic text records before its visibility early-return.
2. The same grapheme layout routine both paints text and produces visible caret/cell geometry. There must not be a second implementation of clipping, width, or offset arithmetic.
3. A framework-owned selection controller keeps two semantic endpoints. It consumes mouse events before ordinary listener broadcast, uses the most recently completed frame's visible text index for hit testing, and requests redraws as necessary.
4. After all components render, a selection pass modifies styles in Ratatui's current `Frame` buffer. Ratatui then performs its normal diff and flush.
5. Copy is generated from semantic records in document order. Visual wrapping and layout gaps do not become copied characters.
6. Arbitrary `Widget` output and direct `buffer_mut()` output are opaque in version 1. They do not become selectable by reverse-engineering cells. An optional semantic widget adapter can be added later.

This design needs a small rendering-state refactor before selection itself: vtui currently propagates offsets, but it does **not** carry an accumulated ancestor clip. Correct nested scrolling and selection cannot be claimed until that is fixed.

## 2. Current architecture (verified)

### 2.1 Public and internal surface

The public construction API is `Component`, `Node`, `Ui`, and `UiNode`; `Component` and `Node` are re-exported through `prelude`, while `Canvas`, `EventContext`, `Ui`, and `UiNode` are re-exported at the crate root (`src/lib.rs`). `LogicalRect`, `Inset`, `Measure`, `Placement`, and `Flow` are public through `prelude` (`src/layout.rs`).

The arena, node IDs, runtime context, transport, drivers, listeners, and input normalization are crate-private. `NodeId` is a slotmap key and is not public (`src/arena.rs`). `NodeAttributes` is also crate-private even though its fields are public within the crate (`src/component.rs`). These are appropriate places for the first selection implementation; exposing them is unnecessary.

`Canvas::buffer_mut()` is public and gives unrestricted access to the Ratatui `Buffer`. Any design that assumes all painting passes through typed Canvas methods would therefore be false.

### 2.2 Mounting and component lifetime

`LaunchBuilder::launch` calls the root factory exactly once and builds a `Runtime` (`src/launch.rs`). `Arena::from` inserts the root, recursively calls each node's composition closure in `remount_subtree`, and computes a pre-order depth-first traversal (`src/arena.rs`). Children retain the order in which `Ui::child` appended them.

At the inspected revision there is no reconciliation loop and no runtime command that remounts a subtree. State changes rerun draw closures, but they do not rerun component factories or composition closures. The component tree and its `NodeId`s are consequently stable for the lifetime of this `Runtime`. The private `remount_subtree` name should not be mistaken for a recurring lifecycle phase; it is used only during initial arena creation.

This contradicts an assumption that vtui already has arbitrary dynamic component insertion/removal. A future reconciler would change the selection identity contract and must explicitly notify/reconcile selection.

### 2.3 Render pipeline and double buffering

The actual path is:

```text
Runtime::draw
  -> ratatui::Terminal::draw(callback)
       -> Arena::render(Frame, Context)
            -> compute_layout for every node
            -> pre-order traversal
                 -> Canvas::new over Frame::buffer_mut()
                 -> node draw closure
       -> Ratatui diff/flush/swap/reset machinery
```

`Arena::render` draws parent before descendants, and siblings in composition order. Later writes overwrite earlier cells. vtui does not allocate, clear, compare, or swap its own next/previous buffers (`src/runtime.rs`, `src/arena.rs`). It renders directly into the mutable buffer supplied by Ratatui's `Frame`. Ratatui 0.30.0 (resolved `ratatui-core` 0.1.2) owns the two buffers: `Terminal::draw` flushes their diff, swaps them, and resets the next buffer.

Selection styling must therefore happen before the callback passed to `Terminal::draw` returns. Applying it after Ratatui diffs would be too late; maintaining a separate vtui buffer would duplicate Ratatui's mechanism.

### 2.4 Layout and the coordinate systems that exist today

The code has two explicit coordinate representations:

- terminal/buffer coordinates: Ratatui `Rect`/`Buffer`, non-negative `u16` positions;
- vtui logical coordinates: `LogicalRect` and `i32` positions, which can be negative or beyond the terminal.

Canvas methods also accept coordinates local to the current component, although there is no named type for that space.

`compute_layout` begins with the terminal frame area converted to `LogicalRect`. For each node it:

1. applies the node's margin to the rectangle allocated by its parent;
2. subtracts the accumulated **ancestor** offset and stores the result as `ArenaNode.rect`;
3. applies the node's padding to the unshifted content rectangle to lay out children;
4. adds the node's own offset to the accumulated offset passed to descendants;
5. splits children in composition order using `Flow`, `Placement`, and `Measure`.

`LogicalRect::with_offset(a, b)` subtracts `(a, b)` from `(x, y)`. A positive offset therefore scrolls/moves content left and up.

For a node whose stored rectangle is `R`, own offset `O`, and local Canvas point `p`, `Canvas::text` maps to the buffer as:

```text
screen = R.origin + p - O
```

Because `R` already has ancestor offsets subtracted, the effective mapping includes all ancestor offsets plus the node's own offset. `Canvas::area()` and `EventContext::area()` discard the stored origin and return `(0, 0, width, height)`.

The current code does not model a separate retained document coordinate system, transform object, viewport stack, or clip stack. Those terms may be useful for the proposed internals, but they are not descriptions of existing types.

### 2.5 Canvas text

`Canvas::text(x, y, text, style)` is the only semantic text operation (`src/canvas.rs`). It accepts `AsRef<str>` and draws one row. It does not accept Ratatui `Text`, `Line`, or `Span`, and it does not wrap or interpret newlines.

Its private `set_stringn`:

- maps local coordinates using the formula above;
- clips vertically and horizontally before iterating graphemes;
- segments extended grapheme clusters with `unicode-segmentation`;
- skips any grapheme containing a control character;
- measures display width with `unicode-width`;
- writes a multi-cell grapheme into the leading Ratatui cell and resets continuation cells;
- skips a whole grapheme at the left clip and stops rather than partially painting one at the right clip.

Thus the current semantic unit most naturally available for endpoint offsets is already an extended grapheme cluster. Bytes remain the best stored offset because Rust strings slice by byte boundaries, but every stored byte offset must be a grapheme boundary. Mouse caret positions map to the boundary before or after a whole grapheme; a wide grapheme cannot be half-selected.

The early vertical/horizontal visibility checks happen before grapheme iteration. Capturing offscreen semantic text must happen before those checks.

### 2.6 Canvas widgets and raw buffers

`Canvas::widget(rect, widget)` receives only an object implementing Ratatui's consuming `Widget` trait. Canvas does not receive the widget's `Text`, spans, wrap decisions, semantic line breaks, or a mapping from cells back to source offsets.

Canvas translates the requested local rectangle, intersects it with the component rectangle and buffer, allocates a temporary buffer with the **entire requested widget width and height**, renders the widget at `(0, 0)`, and copies the visible slice. It exits without invoking the widget if the translated rectangle does not intersect the component rectangle or buffer.

Important consequences:

- arbitrary widget content cannot be made semantic after the fact;
- an offscreen widget does not execute its own `Widget::render`;
- a very large partially visible widget allocates a buffer proportional to its requested logical rectangle, so this path is not an allocation-free infinite canvas;
- Ratatui `Paragraph` owns its wrapping/reflow implementation. vtui cannot recover its source-offset mapping through the `Widget` trait;
- `buffer_mut()` is even more opaque.

The `clipped` attribute is inconsistently applied today. `Canvas::text` clips to the component rectangle only when `clipped == true`; otherwise it clips only to the terminal buffer. `render_widget` always intersects with the component rectangle, regardless of `clipped`. The `Component::set_clipped` documentation says it controls drawing outside visual bounds, but it currently affects only text.

### 2.7 Scrolling, clipping, and virtualization

There is no `ScrollView` type. Scrolling is assembled from `Component::set_offset`, `EventContext::set_offset`, the `SetOffset` command, and `NodeAttributes.offset`. Commands commit after each dispatched event. Parent offsets propagate to descendant positions as of commit `254ac78`.

All component draw closures run every frame, even when their `ArenaNode.rect` is wholly offscreen. A call to `Canvas::text` is still made, but it normally rejects the draw before walking the string. Application code can of course virtualize itself by not issuing offscreen calls; vtui supplies no semantic information for content the application did not emit.

Clipping is not hierarchical. Canvas knows only its own stored rectangle, own attributes, and the terminal buffer. A clipped parent does not pass an effective clip to a child. A descendant with default `clipped == false` can draw text outside an ancestor's rectangle, and a descendant with `clipped == true` clips to its own shifted rectangle, not the ancestor viewport. Nested viewport clipping therefore does not compose today.

The offset is best understood as a content translation whose viewport is intended to remain the node's allocated rectangle: the node's own stored `rect` excludes its own offset, while own rendering and descendants include it. However, descendant `rect`s themselves are shifted, and mouse targeting uses those shifted rectangles. The implementation does not retain an unshifted viewport rectangle needed to express this intent robustly.

### 2.8 Events, mouse targeting, and focus

Crossterm mouse input is normalized to `MouseDown`, `MouseUp`, `MouseHover`, `MouseDrag`, and `MouseScroll` carrying absolute `u16` terminal coordinates (`src/drivers/crossterm.rs`, `src/input.rs`, `src/events.rs`).

For every mouse event, `MouseEvent::target` traverses arena nodes in reverse paint/traversal order and chooses the first stored node rectangle intersecting the one-cell cursor (`src/transport.rs`). It does not consult focusability, drawn cells, `clipped`, ancestor clips, or transparency. Every event is then broadcast to every node in reverse traversal order; `EventContext::is_mouse_hit()` merely compares the current node with that single target. There is no propagation stop. `mouse_coords()` subtracts the stored node rectangle origin; it does not add the node's own content offset.

Focus is a separate `Context.focused: Option<NodeId>`. Components opt in through `set_focusable`; listeners request/resign focus through queued commands. Focus does not automatically follow mouse targeting. Selection should retain this orthogonality.

### 2.9 Tests, examples, and prior selection work

There are currently zero vtui unit/integration tests. `cargo test --all-targets` succeeds but reports no tests. `examples/simple_app.rs` demonstrates an opaque Ratatui `Paragraph` rendered with `Canvas::widget`. Existing invariants can be extracted from it and from the offset-propagation implementation/history, but there are no test helpers to reuse.

Searches of the current tree and reachable Git history found no selection, clipboard, copy, highlight, or experimental selection API. No existing branch establishes selection semantics.

## 3. Requirements and semantic contract

Selection has four deliberately different representations:

| Concern | Representation | Lifetime |
|---|---|---|
| semantic range | anchor and focus semantic endpoints | persistent across frames |
| hit testing | visible grapheme/caret geometry plus nearest-point policy | last completed frame |
| highlight | visible selected cell spans filtered by occlusion | current frame only |
| copy | semantic content and explicit separators in document order | generated on demand/current frame |

A selection is an ordered half-open range between two caret positions. Anchor preserves where the gesture began; focus changes during the drag. Direction is retained for future keyboard extension, but normalization to `(start, end)` is used for painting and copying.

The following are normative:

- Only text emitted as selectable semantic content participates. Component area, focus area, padding, margins, borders, scrollbars, clears, and blank buffer cells are not selectable.
- Spaces contained in semantic text are selectable. Blank rows or columns created only by coordinates/layout are not content.
- Passing through layout whitespace does not break an active selection; the pointer maps to a nearest valid caret.
- Visual wraps do not add copy newlines. Semantic breaks and run-boundary policies do.
- Moving or scrolling content does not alter valid semantic endpoints.
- Only visible, non-occluded cells are highlighted; offscreen content can remain in the semantic range.
- Selection and focus are independent. Applications may separately request focus on mouse-down.
- One gesture is confined to one selection root. Crossing another root clamps to the originating root; it never creates a cross-root range.

## 4. Architecture alternatives

### A. Final-buffer/cell selection

This is simple to prototype and works for opaque widgets currently visible. It is unsuitable as the primary model: Ratatui cells have lost component/run identity, source boundaries, semantic whitespace, document order, offscreen content, and wrap-vs-newline information. A rectangular or screen-row traversal would copy layout gaps and cannot survive scrolling or resize. Wide-cell continuation state and overwritten content further complicate it.

Recommendation: do not expose cell-derived text as first-class selection. A future opt-in “terminal region copy” feature may use it, but it must be named and specified as a different, visual operation.

### B. Component-owned selection

This allows a bespoke component such as an editor to own correct source offsets, virtualization, and copy behavior. It also duplicates gesture handling, nearest-point rules, highlighting, and scrolling integration, and cannot naturally span siblings.

Recommendation: retain an extension/provider interface for virtualized editors, but use a framework controller for ordinary document selection.

### C. Canvas-recorded semantic runs

This exactly matches `Canvas::text`, observes calls before clipping, follows deterministic arena paint traversal, and can share the actual grapheme layout loop. It does not work for `Widget` or raw buffer output and implicit draw-call identity is fragile when conditional calls reorder.

Recommendation: use this as the version-1 semantic source, with explicit stable run keys available where persistence matters.

### D. Retained document/DOM tree

A separate tree could model paragraphs, inline spans, accessibility, mutation, and arbitrary reflow cleanly. vtui currently has a small static component arena and immediate draw closures; adding a second retained hierarchy would duplicate ownership and demand a reconciliation protocol the framework does not otherwise possess.

Recommendation: do not build this for selection. The frame semantic stream should be designed so it could later feed a retained accessibility/document layer without requiring one now.

### E. Hybrid semantic stream plus visible cell ownership (recommended)

Semantic Canvas operations feed a document stream. A viewport-sized visible index supports hits and highlighting. Opaque operations invalidate ownership in the cells they may overwrite. Custom/virtualized components may later provide semantic records directly. This gives semantic correctness for supported content and conservative behavior elsewhere without pretending opaque cells are meaningful.

### F. Two-pass rendering

vtui could first collect semantics and then render. Draw closures may read state, allocate strings, and invoke consuming widgets, so rerunning them risks doubled work and inconsistent side effects. It also does not solve opaque widgets.

Recommendation: one render pass with streaming semantic observation; copy extraction may be a controlled second semantic-only traversal only if a future API makes render callbacks pure enough. Do not rerun current arbitrary draw closures implicitly.

## 5. Recommended internal architecture

All names in this section are illustrative private Rust pseudocode unless explicitly called public.

### 5.1 Rendering state and effective clipping prerequisite

Extend each computed arena node with the information Canvas actually needs:

```rust
// private, illustrative
struct RenderLayout {
    viewport: LogicalRect,       // node's allocated, margin-adjusted viewport on screen
    content_origin: (i32, i32),  // screen origin after ancestor + own content offsets
    effective_clip: LogicalRect, // terminal ∩ every clipping ancestor ∩ this node if clipped
}
```

The exact representation can instead be a translation vector and clip rect. The invariant matters more than names:

> Every visible text cell and every selection hit box is generated from one local-to-screen transform and one effective clip computed by arena layout.

Layout must carry `parent_effective_clip` down the stack. A node that establishes clipping intersects it with its stable viewport; a non-clipping node inherits it. The terminal frame bounds are the root clip. Widget and text paths must honor the same result, resolving today's `clipped` inconsistency. This is a behavior-sensitive refactor and needs tests before selection work.

### 5.2 Roots

Each node is assigned the nearest ancestor selection-root ID during traversal/layout. A root defines ordering and gesture boundaries, not focus or clipping. The arena root is an implicit default selection root, so ordinary `Canvas::text` works without boilerplate. A nested explicit root isolates a document/editor from surrounding UI.

Private identity may use `NodeId`. Public APIs never expose it.

```rust
// private, illustrative
struct RootId(NodeId);
```

Selection can be disabled on a subtree/root for performance or decorative UI. Nested disabled scopes suppress automatic emission until another explicit root begins.

### 5.3 Run and endpoint identity

Document order key:

```rust
// private, illustrative
struct RunAddress {
    root: RootId,
    node: NodeId,
    node_preorder: u32,
    emission: RunKey,
}

enum RunKey {
    Implicit(u32),       // selectable text call ordinal within this node/frame
    Explicit(TextKey),   // user-stable key, scoped to the node
}

struct SelectionPoint {
    run: RunAddress,
    byte: usize,         // guaranteed extended-grapheme boundary
    affinity: Affinity,  // before/after at ambiguous visual boundaries
}
```

`node_preorder` is frame ordering data, not persistent identity. Persistent lookup uses `(root NodeId, node NodeId, RunKey)`; current preorder resolves comparison. `byte` is compact, directly slices UTF-8, and supports source reconstruction. Scalar, display-cell, or wrapped-line offsets are wrong because they may split graphemes or change with layout.

Implicit emission ordinals make existing `Canvas::text` automatically selectable and survive rerenders only while the node's selectable call order remains stable. Conditional/reordered draws should use an explicit `TextKey`. Duplicate explicit keys in one node/frame are a debug assertion and, in release, make the later duplicate nonselectable plus emit diagnostics if vtui gains diagnostics.

A full persistent ID for every draw is not present today. This design adds only scoped optional identity rather than exposing `NodeId` or constructing a DOM.

### 5.4 Frame semantic stream and memory ownership

`Canvas::text` has a borrowed `&str` that may come from a temporary owned `String`; records cannot borrow it after the call. Cloning every emitted string would turn a million offscreen calls into large retained memory.

Use a streaming observer during drawing:

```rust
// private, illustrative
trait TextObserver {
    fn begin_run(&mut self, meta: RunMeta, text: &str) -> Observation;
    fn visible_grapheme(&mut self, placement: GraphemePlacement);
    fn end_run(&mut self);
}
```

The observer:

- retains owned text only for visible runs and the two endpoint runs;
- records lightweight existence/order/length/hash data only as needed to reconcile active endpoints;
- streams selected text into a copy cache when the range is known;
- records viewport-sized visible geometry and ownership;
- can skip all offscreen grapheme segmentation when no active endpoint/copy operation requires inspecting that run.

Idle roots need only visible records so the next mouse-down can hit them. During an active selection, every selectable call still performs O(1) address comparison; text between endpoints is scanned only to produce copy or reconcile content. The current renderer already executes every `Canvas::text` call, but it often rejects offscreen strings before segmentation, so the observer must preserve that fast path when possible.

Because input follows a draw in the runtime loop, hit testing uses an immutable `SelectionFrame` from the last completed render. The next render replaces it atomically after highlighting. It must not contain borrows into draw closure temporaries.

### 5.5 Ordering

Within a root, order is:

1. arena pre-order DFS: parent, then each child subtree in `Ui::child` order;
2. selectable text emission order within each node.

This is deterministic and matches current paint traversal. It deliberately does not sort by `(y, x)`. Fixed/absolute positioning, columns, scrolling, and overlaps do not change semantic order. Drawing a right column before a left column selects right then left unless the component tree/emission order or explicit semantic-order API is changed.

A limitation follows from current rendering: a parent draw closure completes before any child renders, so parent text cannot semantically interleave around a child as HTML inline content can. All parent runs precede all descendant runs. A future need for interleaving is evidence for a richer document API, not a reason to guess from screen position.

Overlapping text remains in document/copy order even when covered. Hit testing chooses the topmost surviving visible run. Highlighting only styles cells owned by the final visible selected run.

### 5.6 Visible geometry, caret mapping, and occlusion

Refactor `set_stringn` around a single grapheme layout iterator that yields:

```rust
// private, illustrative
struct GraphemePlacement {
    source: Range<usize>,       // UTF-8 bytes, one extended grapheme
    screen: LogicalRect,        // width is terminal display width
    clipped_visible: bool,      // whole-grapheme visibility under current policy
    before: usize,
    after: usize,
}
```

Rendering and metadata consume the same yielded placements. This preserves current whole-grapheme clipping unless separately changed and tested.

Maintain an owner token per visible terminal cell, bounded by frame area. `Canvas::text` assigns the run/grapheme token to every cell covered by a painted grapheme. Later semantic writes replace ownership. `Canvas::widget` conservatively invalidates ownership in its copied `final_clip`, because an opaque widget may cover earlier text. Calling `buffer_mut()` invalidates the entire buffer's ownership map for the remainder of that Canvas callback unless a future tracked-buffer wrapper can observe exact writes. These policies prevent selection overlay from highlighting text that later painting hid.

The visible hit index stores caret boundaries around grapheme rectangles, not just glyph cells. It should be grouped by visual row and root; a sorted vector is sufficient at terminal sizes and avoids a general spatial tree.

### 5.7 Mouse policy and margins

Gesture state is framework-owned and processes left-button input before node listeners:

- left `MouseDown` on an exact selectable grapheme or semantic-space cell sets both endpoints and captures that root;
- `MouseDown` on layout-only whitespace does not start selection in version 1. This avoids turning every large root rectangle into an accidental selection target;
- while captured, `MouseDrag` maps exact hits first, otherwise maps to the nearest valid caret in the captured root;
- `MouseUp` performs the same mapping and releases capture;
- leaving the root/terminal clamps to its first or last caret according to pointer position and drag direction; dragging into another root does not switch roots.

Nearest-caret policy for an active drag:

1. Prefer selectable geometry on the pointer's visual row; clamp horizontally to its nearest caret.
2. Otherwise choose the visual row with minimum vertical distance, then the caret with minimum horizontal distance.
3. Break equal geometric distances by the direction from the previous focus (forward chooses the later document caret, backward the earlier one), then document order.
4. Filter candidates to the captured root and effective clip.

This lets a drag cross margins, padding, borders, and blank layout rows without selecting those cells. In a two-column gutter, geometric proximity chooses a column; exact ties remain deterministic and direction-aware. Semantic spaces are exact hit targets because they have source ranges and geometry. Empty semantic lines require a future multiline text primitive to emit a zero-width caret plus a semantic newline; layout-created empty rows emit nothing.

Auto-scroll while dragging is not currently a framework facility. Version 1 can retain a range while application listeners scroll on `MouseDrag`/`MouseScroll`; after the next draw, hit geometry updates and the semantic endpoints remain. Framework auto-scroll requires explicit scroll-container registration and is an open extension.

### 5.8 Selection event integration

Selection should not be implemented as ordinary listeners on every component: mouse events are broadcast, target geometry is component rectangles, and listener order is reverse traversal. Instead, `Runtime::dispatch` or `Arena::update` invokes a private selection controller once before broadcast. It uses the last frame index and may enqueue a redraw/tick if the driver otherwise waits for input.

Ordinary listeners still receive the original event. `is_mouse_hit()` semantics do not change. Mouse selection does not call `request_focus`; an application can do that independently. A future public “selection changed” event should be broadcast only if applications demonstrate a need.

### 5.9 Highlighting

After the final node draw and before `Arena::render` returns:

1. resolve/reconcile semantic endpoints against this frame;
2. normalize the range in document order;
3. inspect only current-frame visible grapheme placements in that range;
4. verify their owner tokens still occupy the cells;
5. patch the leading and continuation Ratatui cells' styles;
6. allow Ratatui's normal diff to observe those style changes.

Default highlight should use `Style::patch`, preferably a configurable background/foreground pair. `REVERSED` is tempting but can interact unpredictably with existing modifiers and terminal defaults. Applying the style to all cells of a wide grapheme, including reset continuation cells, is required for a continuous background. Never rewrite symbols or split a grapheme.

Because the overlay is reapplied to the freshly reset next frame, clearing/changing a selection naturally produces a style diff against Ratatui's previous buffer. No explicit cleanup of the old frame is needed.

### 5.10 Copy semantics

Each semantic run carries source text after the same control/grapheme acceptance policy used by rendering, plus a separator from the preceding run. Copy slices endpoint runs on grapheme-aligned byte offsets, includes full intervening runs, and inserts separators only from semantic metadata.

Initial defaults for the existing one-row `Canvas::text` API:

- spaces and other printable graphemes are preserved exactly;
- control-containing graphemes are excluded, matching current rendering (tabs/newlines are currently invisible and therefore not silently copied);
- distinct `Canvas::text` calls are block-like and default to a newline separator;
- crossing a component boundary also defaults to one newline, coalesced with the run separator so it does not create two newlines;
- visual coordinate gaps add nothing;
- visual wrapping adds nothing;
- style spans, once supported, concatenate with no separator inside their owning text block;
- clipping and scrolling do not remove selected source text;
- root boundaries cannot be crossed.

The newline default is deterministic but may not match applications that emulate inline styled spans with adjacent `text` calls. Such callers need explicit join metadata; see the API proposal. A multiline/rich-text primitive should retain source newline information and distinguish it from wrapping. Tabs should be specified by that future primitive rather than retroactively copied from currently invisible controls.

The controller should produce a `String`. Sending it to an OS clipboard is driver/platform policy and is out of scope for the semantic core. A public `selected_text()` accessor or copy-request event can be added before clipboard integration.

### 5.11 Rerender reconciliation

On every frame:

- if both `(NodeId, RunKey)` endpoints reappear, clamp byte offsets to the nearest preceding grapheme boundary and then to string length;
- explicit keys survive emission reordering and layout changes; implicit keys mean “same ordinal in same node” and may bind to changed content;
- compare a lightweight text fingerprint for implicit runs. If the endpoint run's content changes, clear selection rather than silently selecting unrelated content in version 1;
- for explicit keys, content changes preserve/clamp endpoints because the caller asserted semantic identity;
- if either endpoint run or root disappears, clear the selection;
- if current preorder reverses/reorders endpoints, retain anchor/focus identities and recompute the normalized range.

In the current static arena, nodes do not disappear or reorder after mount, but run calls and strings can change with state. These rules also define a safe baseline for a future reconciler. Browser-level mutation tracking is explicitly not promised.

## 6. Minimal public API proposal

Existing APIs remain unchanged and `Canvas::text` automatically emits block-like selectable content under the implicit root.

### 6.1 Root/boundary configuration

```rust
// proposed public Component methods
impl Component {
    pub fn set_selection_root(&self, enabled: bool);
    pub fn set_selectable(&self, selectable: bool);
}
```

`set_selection_root(true)` is public because applications need isolation between documents/editors. It is a node attribute configured during factory construction, like `set_focusable` and `set_clipped`; it owns nothing and has no lifetime parameter. The arena root behaves as though it were a root even without this call.

`set_selectable(false)` is public because large/decorative subtrees need opt-out and because automatic selection of all `Canvas::text` output is otherwise too broad. It suppresses inherited participation for the subtree. Naming should be revisited to avoid confusion with a root; an enum attribute may ultimately be clearer.

### 6.2 Stable text keys and semantics

```rust
// proposed public value types; exact names are provisional
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TextKey(u64);

#[derive(Clone, Copy, Debug, Default)]
pub enum TextJoin {
    Inline,
    #[default]
    Line,
    Paragraph,
    None,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TextOptions {
    pub key: Option<TextKey>,
    pub join: TextJoin,
    pub selectable: bool,
}

impl Canvas<'_> {
    pub fn text_with(
        &mut self,
        x: i32,
        y: i32,
        text: impl AsRef<str>,
        style: impl Into<Style>,
        options: TextOptions,
    );
}
```

`TextKey` is public only because conditional draw order otherwise cannot promise persistent endpoints. It is scoped to the current component, so a simple integer is enough and it owns no allocation. A string key could be added later if ergonomics warrant it.

`TextJoin`/`TextOptions` are public because Canvas cannot infer whether separate calls are inline spans, lines, paragraphs, or decorative. `Paragraph` can map to `"\n\n"`; `Line` to `"\n"`; `Inline`/`None` need careful naming (`None` may mean no preceding separator, not nonselectable). Before implementation, prefer builder methods or a dedicated `SemanticText` argument if booleans/defaults prove ambiguous.

Example:

```rust
c.set_selection_root(true);
c.draw(|canvas| {
    canvas.text_with(
        4,
        2,
        "First paragraph.",
        Style::default(),
        TextOptions {
            key: Some(TextKey::new(1)),
            join: TextJoin::Paragraph,
            selectable: true,
        },
    );
});
```

The constructors/default fields must be adjusted to Rust privacy and forward-compatibility conventions in implementation; the sketch communicates semantics, not a committed signature.

### 6.3 Copy access

Do not initially expose `Selection`, `SelectionPoint`, `NodeId`, geometry, or the frame index. Add only one application-facing query/action when copy integration lands:

```rust
// one possible proposed API, not both
event.selected_text() -> Option<&str>;
event.copy_selection();
```

The correct placement depends on clipboard ownership (runtime driver versus application). Keeping this internal in the first highlight-only milestone is preferable. Custom editor/provider traits should also remain private/experimental until a real component exercises them.

## 7. Exact proposed pipeline

```text
Ratatui Terminal::draw begins with reset next buffer
        |
        v
Arena computes node transforms + accumulated clips + root membership
        |
        v
pre-order node rendering
        |
        +-- Canvas::text
        |      |
        |      +-- assign (root, node, key/ordinal) before visibility rejection
        |      +-- semantic observer sees text/join metadata
        |      +-- shared grapheme layout maps local -> screen and clips
        |      +-- visible glyphs paint buffer and update owner/hit index
        |
        +-- Canvas::widget / buffer_mut
               +-- normal painting
               +-- invalidate covered ownership conservatively
        |
        v
reconcile persistent semantic endpoints against current emission stream
        |
        v
selection overlay patches styles of visible, selected, still-owned cells
        |
        v
publish immutable SelectionFrame for the next input update
        |
        v
return from callback; Ratatui diffs, flushes, swaps, resets

Later mouse input
        |
        v
selection controller exact/nearest hit-tests published SelectionFrame
        |
        +-- updates semantic anchor/focus only
        +-- ordinary reverse-order listener broadcast remains unchanged
        |
        v
next draw projects the range at new scroll/layout coordinates
```

Copy generation consumes semantic source ranges, never the styled final buffer.

## 8. Edge-case decisions

### Margins, padding, borders, and layout blanks

They have no semantic records and never appear in copied text or highlight. They map an active drag to a nearby caret. A drag does not start there in version 1. Focus rectangles are irrelevant.

### Wrapping

Existing `Canvas::text` does not wrap. Arbitrary Ratatui `Paragraph` is opaque, and its public `Widget` interface does not expose the private reflow-to-source map used during render. Version 1 therefore does **not** promise semantic selection for `Paragraph` passed through `Canvas::widget`.

Add a vtui-owned `Canvas::paragraph`/rich-text operation later. It must accept semantic `Text`/lines, run the same wrapping algorithm for rendering and placements, retain source byte ranges for every wrapped grapheme, and record source newlines separately. Resize recomputes placements but leaves keyed source offsets unchanged. Copy omits soft-wrap breaks.

### Clipping and wide glyphs

Mirror current all-or-nothing grapheme painting at both horizontal edges. A click in a continuation cell maps to the same grapheme; nearest half chooses before/after by the midpoint, with a deterministic affinity for even widths. Highlight all display cells. Combining marks remain in their extended grapheme and cannot be individually addressed.

The current left-clip `skip_width` logic should receive focused tests; clipping through a wide grapheme must not cause source/screen mapping drift. Any correction must be shared by painting and selection.

### Nested scrolling

Not correct until accumulated clips and stable viewports are introduced. Once they are, transforms add and clips intersect at every ancestor. Semantic order ignores scroll nesting. Visible hit/highlight uses the final intersection; endpoints/copy use run identity/source offsets and remain valid offscreen.

### Offscreen content and virtualization

Offscreen `Canvas::text` calls are semantically observable because component renderers execute, provided observation happens before Canvas's early return. Content not emitted due to application virtualization does not exist to this mechanism. A virtualized document must eventually implement a provider that can resolve a semantic key, compare positions, and extract ranges without drawing all items.

### Styled spans

The current semantic API has one style per string. A future rich-text operation should give one logical run identity to the whole source and treat spans as style subranges, not separate default-newline runs. Selection overlay patches each cell's existing style, preserving non-selection attributes according to `Style::patch` precedence.

### Overlap

Document order remains traversal/emission order. Exact hits and visual highlighting use final cell ownership, so a covered run cannot capture a click or shine through its cover. Copying a range that semantically includes an overlapped run still includes it, just as CSS overlap does not remove DOM text.

### Dynamic text and trees

Implicit keyed text changes clear endpoint selections; explicit keys preserve and clamp. Missing endpoints clear. Current node trees are static, but these rules are mandatory if reconciliation is introduced.

### Empty text

An empty one-row `Canvas::text` contributes no hit geometry and should not create a selectable run unless it carries a semantic separator between nonempty runs. Future empty semantic lines need a caret representation.

## 9. Performance analysis

Let `D` be selectable draw calls, `V` visible terminal cells, `Gv` visible graphemes, `S` bytes copied, and `Ge` graphemes scanned in endpoint/selected runs.

Baseline rendering already invokes O(D) Canvas calls and allocates O(V) in Ratatui. The proposed idle overhead is:

- time: O(D) small run-address checks plus O(Gv) placements already needed for visible text;
- memory: O(V) owner tokens plus O(Gv) hit records and owned copies of visible text;
- no full logical cell buffer and no retained copy of all offscreen strings.

With an active range/copy:

- reconciliation is O(D);
- extraction is O(Ge + S) and the resulting copied string is necessarily O(S);
- no offscreen cell geometry is retained.

For one million offscreen `canvas.text` calls, selection adds one cheap observer call/address increment per draw but must preserve Canvas's pre-grapheme visibility rejection. It must not allocate a run object or clone text per call in idle mode. If an active selection spans those calls, producing the copied document is inherently linear; retaining a million-record catalog is still avoidable with streaming comparisons and explicit keys. Benchmark this case before enabling implicit root capture globally.

`Canvas::widget` already violates the strongest “infinite canvas” performance claim by allocating the entire widget rectangle when partially visible. Selection must not worsen it, and a separate widget-render clipping redesign may be warranted.

Potential optimizations, only after measurement:

- compact owner indices (`u32`) instead of full addresses per cell;
- row-sorted hit spans rather than per-cell hit objects;
- disable semantic observation in subtrees with `set_selectable(false)`;
- cache grapheme boundaries for explicit keyed immutable text using fingerprint + width;
- provider APIs for virtualized documents.

## 10. Implementation plan

### Stage 0: lock down current rendering

Add `#[cfg(test)]` tests around `Arena`, `Canvas`, and a Ratatui `TestBackend`/`Buffer` for traversal paint order, margin/padding, positive/negative offsets, inherited offsets, text/widget clipping differences, wide graphemes, and event targeting. This makes current quirks explicit before changing them.

Likely files: `src/arena.rs`, `src/canvas.rs`, `src/layout.rs`, `src/transport.rs`; new internal test helpers.

### Stage 1: unify transform and clip state

Compute stable node viewport, content translation, and accumulated effective clip in layout. Make text and widget rendering consume the same state. Add nested-offset/nested-clip tests. Decide whether fixing `set_clipped(false)` widget behavior is a bug fix or documented breaking change.

Likely files: `src/arena.rs`, `src/canvas.rs`, `src/layout.rs`, `src/component.rs` docs.

### Stage 2: shared grapheme placement iterator

Extract Canvas text layout so one iterator drives painting and reports byte ranges/cell rectangles. Preserve or deliberately correct clipping behavior with golden buffer tests. No selection behavior yet.

Likely file: `src/canvas.rs`; optionally new private `src/text.rs`.

### Stage 3: visible semantic frame

Add private root membership, implicit run ordinals, visible caret geometry, and cell ownership. Instrument `widget`/`buffer_mut` as conservative occluders. Test exact hits, overlap, opaque output, Unicode, and no offscreen allocations.

Likely files: new `src/selection.rs`, `src/canvas.rs`, `src/arena.rs`, `src/lib.rs` module declaration.

### Stage 4: gesture controller and highlight

Store anchor/focus in `Context` or a dedicated runtime-owned controller, intercept mouse events once, implement nearest-caret behavior, and apply the post-render style overlay. Test forward/backward drags, margins, roots, focus independence, scrolling, and Ratatui diffs.

Likely files: `src/selection.rs`, `src/runtime.rs` or `src/arena.rs`, `src/context.rs`, `src/transport.rs`, `src/canvas.rs`.

### Stage 5: stable keys, separators, and copy string

Add the minimal `TextKey`/options API, endpoint reconciliation, streamed extraction, and an internal copy result. Test conditional emissions, text mutation, separators, clipping, and offscreen ranges. Keep OS clipboard integration separate.

Likely files: `src/selection.rs`, `src/canvas.rs`, `src/component.rs`, `src/lib.rs` exports.

### Stage 6: public copy integration and example

Choose driver/application clipboard ownership, expose the smallest accessor/action, and add an interactive selection example with scrolling and nested roots. Do not claim arbitrary widgets or wrapped paragraphs yet.

### Later stages

Add a vtui semantic paragraph/rich-text API with source-aware wrapping, then experiment with a provider trait for virtualized editors and semantic widgets. Cell-based fallback should remain a separately named visual-copy mode if added at all.

## 11. Test plan

### Unit tests

Selection model:

- part of one run; empty/collapsed range;
- forward and backward normalization;
- multiple runs and components in pre-order/emission order;
- explicit versus implicit keys, duplicates, disappearance, reorder, content changes, and clamping;
- line/paragraph/inline separators and component boundaries;
- grapheme-aligned byte slicing for ASCII, CJK wide characters, emoji ZWJ/variation sequences, and combining marks;
- semantic spaces versus skipped controls and layout whitespace;
- root containment and disabled subtrees.

Geometry:

- exact glyph/space hit;
- before/after caret halves for one- and two-cell graphemes;
- start in margin rejected;
- active drag through/end in margins maps nearest;
- blank rows, borders, two-column gutters, outside-root clamp;
- clipping at all edges and no half-wide glyph;
- overlap and widget/raw-buffer occlusion;
- nested translation and effective-clip intersection.

### Integration tests

Use Ratatui `TestBackend` or construct `Buffer`/`Frame` through the runtime test seam:

- selection across sibling and nested components;
- focusable without text, selectable without focus, both, neither;
- selection does not change `is_mouse_hit` or focus;
- scroll offset changes projection but not endpoint identity;
- endpoints outside the visible viewport remain copied;
- scroll while dragging and while selection is idle;
- nested scroll clips and projections;
- resize/reflow once semantic paragraph exists;
- styled runs preserve styles under selection and restore through next-buffer reset/diff;
- clearing/changing selection produces correct Ratatui buffer differences;
- later painters hide earlier selectable text;
- decorative widget glyphs remain excluded;
- one-million-offscreen-draw benchmark/allocation assertion.

There are no existing vtui tests to reuse. Reuse implementation invariants from `Canvas::set_stringn`, `compute_layout`, `compute_traversal`, `MouseEvent::target`, the offset-propagation commits, and Ratatui's own `BufferDiff`/wide-cell test patterns.

### Interactive examples

Add one example containing paragraphs separated by padding, two columns, an explicit nested root, a large offset-driven scroll region, mixed styles, wide/combining Unicode, opaque widgets, and focusable controls. Display the framework-generated copy string in a side panel so semantics can be checked without platform clipboard dependencies. A second stress example should scroll through many emitted lines and report selection-index allocations/time.

## 12. Open questions requiring product/API decisions

1. **Margin-start behavior.** This spec rejects starting in layout whitespace but maps whitespace during an active drag. If strict browser parity requires margin-start selection, roots need an explicit policy and likely a maximum/row-based snap rule.
2. **Default scope.** Is the arena root implicitly selectable, or should applications explicitly create a root? Implicit is ergonomic; explicit avoids overhead/accidental label selection.
3. **Existing `Canvas::text` default separator.** Newline is deterministic and matches its one-row nature, but adjacent calls may currently emulate inline spans. Usage data should decide whether default is `Line` or `Inline`.
4. **`clipped` semantics.** Current text and widget behavior disagree. Selection requires one rule, but changing widget overflow may be observable.
5. **Selection style.** Fixed theme, launch-builder configuration, per-root configuration, and terminal capability fallback need a decision.
6. **Copy trigger and clipboard owner.** vtui can produce a `String`; whether Ctrl-C, OSC 52, a driver API, or application code transfers it is separate and security/platform sensitive.
7. **Autoscroll.** There is no scroll-container abstraction to command when dragging beyond an edge. An offset alone does not identify axis, bounds, or owner.
8. **Virtualized providers.** Applications that do not emit offscreen content need an explicit semantic document/provider contract. Its API should be designed with a real editor/list component.
9. **Opaque widget adoption.** Prefer a vtui semantic paragraph API or an extension trait implemented by cooperating widgets; do not promise generic `Widget` introspection. Which comes first depends on desired Ratatui compatibility.
10. **Dynamic reconciliation roadmap.** Current trees are static. If a reconciler is planned, stable component identity should be designed jointly with selection rather than layering public selection IDs over temporary `NodeId`s.
11. **Parent/child interleaving.** Current pre-order rendering cannot express parent text after a child in document order. A future semantic order override or document builder may be needed.
12. **Control and tab policy.** Current text silently skips controls. A new multiline API must define tab expansion and source-preserving newlines explicitly.

## 13. Acceptance invariants

An implementation conforms to this design only if all of these remain true:

- persistent endpoints contain semantic identity and grapheme-boundary source offsets, never only screen coordinates;
- focus rectangles and layout whitespace never manufacture selectable content;
- the exact transform/clip/grapheme placement used for painting supplies hit and highlight geometry;
- selection across margins changes only endpoint projection, not copied content;
- offscreen selected text requires no offscreen Ratatui cell buffer;
- opaque widget/raw-buffer output is either explicitly annotated or conservatively nonselectable, never silently presented as semantic text;
- highlighting is applied to the current Ratatui frame before diffing and only to final visible owners;
- document order is deterministic pre-order plus emission order unless an explicit future ordering API says otherwise;
- nested scrolling is not declared supported until ancestor clip composition is implemented and tested;
- performance remains O(viewport) in retained geometry and does not clone every offscreen string by default.
