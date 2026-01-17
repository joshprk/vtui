use std::collections::HashMap;

use itertools::Itertools;
use kasuari::{AddConstraintError, Expression, Solver, Strength, Variable, WeightedRelation::{EQ, GE, LE}};

use crate::{canvas::LogicalRect, component::Node, layout::strengths::{ALL_SEGMENT_GROW, FILL_GROW, GROW, LENGTH_SIZE_EQ, MAX_SIZE_EQ, MAX_SIZE_LE, MIN_SIZE_EQ, MIN_SIZE_GE, PERCENTAGE_SIZE_EQ, RATIO_SIZE_EQ, SPACE_GROW, SPACER_SIZE_EQ}};

const FLOAT_PRECISION_MULTIPLIER: f64 = 100.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NodeAxis {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeConstraint {
    Min(i32),
    Max(i32),
    Length(i32),
    Percentage(u16),
    Ratio(u32, u32),
    Fill(u16),
}

impl Default for NodeConstraint {
    fn default() -> Self {
        Self::Percentage(100)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeMargin {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeSpacing {
    Space(i32),
    Overlap(i32),
}

impl Default for NodeSpacing {
    fn default() -> Self {
        Self::Space(0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NodeFlex {
    Legacy,
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeLayout {
    pub axis: NodeAxis,
    pub constraint: NodeConstraint,
    pub margin: NodeMargin,
    pub spacing: NodeSpacing,
    pub flex: NodeFlex,
}

impl NodeLayout {
    pub fn split(
        &self,
        rect: LogicalRect,
        children: Vec<&Node>,
    ) -> Vec<LogicalRect> {
        self.split_with_spacers(rect, children).0
    }

    pub fn split_with_spacers(
        &self,
        rect: LogicalRect,
        children: Vec<&Node>,
    ) -> (Vec<LogicalRect>, Vec<LogicalRect>) {
        self.try_split(rect, children).unwrap()
    }
}

impl NodeLayout {
    fn try_split(
        &self,
        rect: LogicalRect,
        children: Vec<&Node>,
    ) -> Result<(Vec<LogicalRect>, Vec<LogicalRect>), AddConstraintError> {
        let mut solver = Solver::new();
        let inner_area = rect.inner(self.margin);

        let (area_start, area_end) = match self.axis {
            NodeAxis::Horizontal => (
                f64::from(inner_area.x) * FLOAT_PRECISION_MULTIPLIER,
                f64::from(inner_area.right()) * FLOAT_PRECISION_MULTIPLIER,
            ),
            NodeAxis::Vertical => (
                f64::from(inner_area.y) * FLOAT_PRECISION_MULTIPLIER,
                f64::from(inner_area.bottom()) * FLOAT_PRECISION_MULTIPLIER,
            ),
        };

        let variable_count = children.len() * 2 + 2;

        let variables = std::iter::repeat_with(Variable::new)
            .take(variable_count)
            .collect_vec();

        let spacers = variables
            .iter()
            .tuples()
            .map(|(a, b)| Element::from((*a, *b)))
            .collect_vec();

        let segments = variables
            .iter()
            .skip(1)
            .tuples()
            .map(|(a, b)| Element::from((*a, *b)))
            .collect_vec();

        let flex = self.flex;

        let spacing = match self.spacing {
            NodeSpacing::Overlap(x) => -x,
            NodeSpacing::Space(x) => x,
        };

        let constraints = children.iter()
            .map(|child| child.layout.constraint)
            .collect_vec();

        let area_size = Element::from((*variables.first().unwrap(), *variables.last().unwrap()));

        configure_area(&mut solver, area_size, area_start, area_end)?;
        configure_variable_in_area_constraints(&mut solver, &variables, area_size)?;
        configure_variable_constraints(&mut solver, &variables)?;
        configure_flex_constraints(&mut solver, area_size, &spacers, flex, spacing)?;
        configure_constraints(&mut solver, area_size, &segments, &constraints, flex)?;
        configure_fill_constraints(&mut solver, &segments, &constraints, flex)?;

        if flex != NodeFlex::Legacy {
            for (left, right) in segments.iter().tuple_windows() {
                solver.add_constraint(left.has_size(right, ALL_SEGMENT_GROW))?;
            }
        }

        // `solver.fetch_changes()` can only be called once per solve
        let changes: HashMap<Variable, f64> = solver.fetch_changes().iter().copied().collect();

        let segment_rects = changes_to_rects(&changes, &segments, inner_area, self.axis);
        let spacer_rects = changes_to_rects(&changes, &spacers, inner_area, self.axis);

        Ok((segment_rects, spacer_rects))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct Element {
    start: Variable,
    end: Variable,
}

impl From<(Variable, Variable)> for Element {
    fn from((start, end): (Variable, Variable)) -> Self {
        Self { start, end }
    }
}

impl From<Element> for Expression {
    fn from(element: Element) -> Self {
        element.size()
    }
}

impl From<&Element> for Expression {
    fn from(element: &Element) -> Self {
        element.size()
    }
}

impl Element {
    fn size(&self) -> Expression {
        self.end - self.start
    }

    fn has_max_size(&self, size: i32, strength: Strength) -> kasuari::Constraint {
        self.size() | LE(strength) | (f64::from(size) * FLOAT_PRECISION_MULTIPLIER)
    }

    fn has_min_size(&self, size: i32, strength: Strength) -> kasuari::Constraint {
        self.size() | GE(strength) | (f64::from(size) * FLOAT_PRECISION_MULTIPLIER)
    }

    fn has_int_size(&self, size: i32, strength: Strength) -> kasuari::Constraint {
        self.size() | EQ(strength) | (f64::from(size) * FLOAT_PRECISION_MULTIPLIER)
    }

    fn has_size<E: Into<Expression>>(&self, size: E, strength: Strength) -> kasuari::Constraint {
        self.size() | EQ(strength) | size.into()
    }

    fn has_double_size<E: Into<Expression>>(
        &self,
        size: E,
        strength: Strength,
    ) -> kasuari::Constraint {
        self.size() | EQ(strength) | (size.into() * 2.0)
    }

    fn is_empty(&self) -> kasuari::Constraint {
        self.size() | EQ(Strength::REQUIRED - Strength::WEAK) | 0.0
    }
}

fn configure_area(
    solver: &mut Solver,
    area: Element,
    area_start: f64,
    area_end: f64,
) -> Result<(), AddConstraintError> {
    solver.add_constraint(area.start | EQ(Strength::REQUIRED) | area_start)?;
    solver.add_constraint(area.end | EQ(Strength::REQUIRED) | area_end)?;
    Ok(())
}

fn configure_variable_in_area_constraints(
    solver: &mut Solver,
    variables: &[Variable],
    area: Element,
) -> Result<(), AddConstraintError> {
    // all variables are in the range [area.start, area.end]
    for &variable in variables {
        solver.add_constraint(variable | GE(Strength::REQUIRED) | area.start)?;
        solver.add_constraint(variable | LE(Strength::REQUIRED) | area.end)?;
    }

    Ok(())
}

fn configure_variable_constraints(
    solver: &mut Solver,
    variables: &[Variable],
) -> Result<(), AddConstraintError> {
    for (&left, &right) in variables.iter().skip(1).tuples() {
        solver.add_constraint(left | LE(Strength::REQUIRED) | right)?;
    }
    Ok(())
}

fn configure_constraints(
    solver: &mut Solver,
    area: Element,
    segments: &[Element],
    constraints: &[NodeConstraint],
    flex: NodeFlex,
) -> Result<(), AddConstraintError> {
    for (&constraint, &segment) in constraints.iter().zip(segments.iter()) {
        match constraint {
            NodeConstraint::Max(max) => {
                solver.add_constraint(segment.has_max_size(max, MAX_SIZE_LE))?;
                solver.add_constraint(segment.has_int_size(max, MAX_SIZE_EQ))?;
            }
            NodeConstraint::Min(min) => {
                solver.add_constraint(segment.has_min_size(min, MIN_SIZE_GE))?;
                if flex == NodeFlex::Legacy {
                    solver.add_constraint(segment.has_int_size(min, MIN_SIZE_EQ))?;
                } else {
                    solver.add_constraint(segment.has_size(area, FILL_GROW))?;
                }
            }
            NodeConstraint::Length(length) => {
                solver.add_constraint(segment.has_int_size(length, LENGTH_SIZE_EQ))?;
            }
            NodeConstraint::Percentage(p) => {
                let size = area.size() * f64::from(p) / 100.00;
                solver.add_constraint(segment.has_size(size, PERCENTAGE_SIZE_EQ))?;
            }
            NodeConstraint::Ratio(num, den) => {
                // avoid division by zero by using 1 when denominator is 0
                let size = area.size() * f64::from(num) / f64::from(den.max(1));
                solver.add_constraint(segment.has_size(size, RATIO_SIZE_EQ))?;
            }
            NodeConstraint::Fill(_) => {
                // given no other constraints, this segment will grow as much as possible.
                solver.add_constraint(segment.has_size(area, FILL_GROW))?;
            }
        }
    }
    Ok(())
}

fn configure_flex_constraints(
    solver: &mut Solver,
    area: Element,
    spacers: &[Element],
    flex: NodeFlex,
    spacing: i32,
) -> Result<(), AddConstraintError> {
    let spacers_except_first_and_last = spacers.get(1..spacers.len() - 1).unwrap_or(&[]);
    let spacing_f64 = f64::from(spacing) * FLOAT_PRECISION_MULTIPLIER;
    match flex {
        NodeFlex::Legacy => {
            for spacer in spacers_except_first_and_last {
                solver.add_constraint(spacer.has_size(spacing_f64, SPACER_SIZE_EQ))?;
            }
            if let (Some(first), Some(last)) = (spacers.first(), spacers.last()) {
                solver.add_constraint(first.is_empty())?;
                solver.add_constraint(last.is_empty())?;
            }
        }

        // All spacers excluding first and last are the same size and will grow to fill
        // any remaining space after the constraints are satisfied.
        // All spacers excluding first and last are also twice the size of the first and last
        // spacers
        NodeFlex::SpaceAround => {
            if spacers.len() <= 2 {
                // If there are two or less spacers, fallback to NodeFlex::SpaceEvenly
                for (left, right) in spacers.iter().tuple_combinations() {
                    solver.add_constraint(left.has_size(right, SPACER_SIZE_EQ))?;
                }
                for spacer in spacers {
                    solver.add_constraint(spacer.has_min_size(spacing, SPACER_SIZE_EQ))?;
                    solver.add_constraint(spacer.has_size(area, SPACE_GROW))?;
                }
            } else {
                // Separate the first and last spacer from the middle ones
                let (first, rest) = spacers.split_first().unwrap();
                let (last, middle) = rest.split_last().unwrap();

                // All middle spacers should be equal in size
                for (left, right) in middle.iter().tuple_combinations() {
                    solver.add_constraint(left.has_size(right, SPACER_SIZE_EQ))?;
                }

                // First and last spacers should be half the size of any middle spacer
                if let Some(first_middle) = middle.first() {
                    solver.add_constraint(first_middle.has_double_size(first, SPACER_SIZE_EQ))?;
                    solver.add_constraint(first_middle.has_double_size(last, SPACER_SIZE_EQ))?;
                }

                // Apply minimum size and growth constraints
                for spacer in spacers {
                    solver.add_constraint(spacer.has_min_size(spacing, SPACER_SIZE_EQ))?;
                    solver.add_constraint(spacer.has_size(area, SPACE_GROW))?;
                }
            }
        }

        // All spacers are the same size and will grow to fill any remaining space after the
        // constraints are satisfied
        NodeFlex::SpaceEvenly => {
            for (left, right) in spacers.iter().tuple_combinations() {
                solver.add_constraint(left.has_size(right, SPACER_SIZE_EQ))?;
            }
            for spacer in spacers {
                solver.add_constraint(spacer.has_min_size(spacing, SPACER_SIZE_EQ))?;
                solver.add_constraint(spacer.has_size(area, SPACE_GROW))?;
            }
        }

        // All spacers excluding first and last are the same size and will grow to fill
        // any remaining space after the constraints are satisfied.
        // The first and last spacers are zero size.
        NodeFlex::SpaceBetween => {
            for (left, right) in spacers_except_first_and_last.iter().tuple_combinations() {
                solver.add_constraint(left.has_size(right.size(), SPACER_SIZE_EQ))?;
            }
            for spacer in spacers_except_first_and_last {
                solver.add_constraint(spacer.has_min_size(spacing, SPACER_SIZE_EQ))?;
                solver.add_constraint(spacer.has_size(area, SPACE_GROW))?;
            }
            if let (Some(first), Some(last)) = (spacers.first(), spacers.last()) {
                solver.add_constraint(first.is_empty())?;
                solver.add_constraint(last.is_empty())?;
            }
        }

        NodeFlex::Start => {
            for spacer in spacers_except_first_and_last {
                solver.add_constraint(spacer.has_size(spacing_f64, SPACER_SIZE_EQ))?;
            }
            if let (Some(first), Some(last)) = (spacers.first(), spacers.last()) {
                solver.add_constraint(first.is_empty())?;
                solver.add_constraint(last.has_size(area, GROW))?;
            }
        }
        NodeFlex::Center => {
            for spacer in spacers_except_first_and_last {
                solver.add_constraint(spacer.has_size(spacing_f64, SPACER_SIZE_EQ))?;
            }
            if let (Some(first), Some(last)) = (spacers.first(), spacers.last()) {
                solver.add_constraint(first.has_size(area, GROW))?;
                solver.add_constraint(last.has_size(area, GROW))?;
                solver.add_constraint(first.has_size(last, SPACER_SIZE_EQ))?;
            }
        }
        NodeFlex::End => {
            for spacer in spacers_except_first_and_last {
                solver.add_constraint(spacer.has_size(spacing_f64, SPACER_SIZE_EQ))?;
            }
            if let (Some(first), Some(last)) = (spacers.first(), spacers.last()) {
                solver.add_constraint(last.is_empty())?;
                solver.add_constraint(first.has_size(area, GROW))?;
            }
        }
    }
    Ok(())
}

fn configure_fill_constraints(
    solver: &mut Solver,
    segments: &[Element],
    constraints: &[NodeConstraint],
    flex: NodeFlex,
) -> Result<(), AddConstraintError> {
    for ((&left_constraint, &left_segment), (&right_constraint, &right_segment)) in constraints
        .iter()
        .zip(segments.iter())
        .filter(|(c, _)|
            matches!(c, NodeConstraint::Fill(..)) ||
                (flex != NodeFlex::Legacy && matches!(c, NodeConstraint::Min(..))
        ))
        .tuple_combinations()
    {
        let left_scaling_factor = match left_constraint {
            NodeConstraint::Fill(scale) => f64::from(scale).max(1e-6),
            NodeConstraint::Min(_) => 1.0,
            _ => unreachable!(),
        };
        let right_scaling_factor = match right_constraint {
            NodeConstraint::Fill(scale) => f64::from(scale).max(1e-6),
            NodeConstraint::Min(_) => 1.0,
            _ => unreachable!(),
        };
        solver.add_constraint(
            (right_scaling_factor * left_segment.size())
                | EQ(GROW)
                | (left_scaling_factor * right_segment.size()),
        )?;
    }
    Ok(())
}

fn changes_to_rects(
    changes: &HashMap<Variable, f64>,
    elements: &[Element],
    area: LogicalRect,
    direction: NodeAxis,
) -> Vec<LogicalRect> {
    elements
        .iter()
        .map(|element| {
            let start = changes.get(&element.start).unwrap_or(&0.0);
            let end = changes.get(&element.end).unwrap_or(&0.0);
            let start = round(round(*start) / FLOAT_PRECISION_MULTIPLIER) as i32;
            let end = round(round(*end) / FLOAT_PRECISION_MULTIPLIER) as i32;
            let size = end.saturating_sub(start);
            match direction {
                NodeAxis::Horizontal => LogicalRect {
                    x: start,
                    y: area.y,
                    width: size,
                    height: area.height,
                },
                NodeAxis::Vertical => LogicalRect {
                    x: area.x,
                    y: start,
                    width: area.width,
                    height: size,
                },
            }
        })
        .collect::<Vec<LogicalRect>>()
}

fn round(value: f64) -> f64 {
    value.round()
}

mod strengths {
    use kasuari::Strength;

    /// The strength to apply to Spacers to ensure that their sizes are equal.
    ///
    /// ┌     ┐┌───┐┌     ┐┌───┐┌     ┐
    ///   ==x  │   │  ==x  │   │  ==x
    /// └     ┘└───┘└     ┘└───┘└     ┘
    pub const SPACER_SIZE_EQ: Strength = Strength::REQUIRED.div_f64(10.0);

    /// The strength to apply to Min inequality constraints.
    ///
    /// ┌────────┐
    /// │Min(>=x)│
    /// └────────┘
    pub const MIN_SIZE_GE: Strength = Strength::STRONG.mul_f64(100.0);

    /// The strength to apply to Max inequality constraints.
    ///
    /// ┌────────┐
    /// │Max(<=x)│
    /// └────────┘
    pub const MAX_SIZE_LE: Strength = Strength::STRONG.mul_f64(100.0);

    /// The strength to apply to Length constraints.
    ///
    /// ┌───────────┐
    /// │Length(==x)│
    /// └───────────┘
    pub const LENGTH_SIZE_EQ: Strength = Strength::STRONG.mul_f64(10.0);

    /// The strength to apply to Percentage constraints.
    ///
    /// ┌───────────────┐
    /// │Percentage(==x)│
    /// └───────────────┘
    pub const PERCENTAGE_SIZE_EQ: Strength = Strength::STRONG;

    /// The strength to apply to Ratio constraints.
    ///
    /// ┌────────────┐
    /// │Ratio(==x,y)│
    /// └────────────┘
    pub const RATIO_SIZE_EQ: Strength = Strength::STRONG.div_f64(10.0);

    /// The strength to apply to Min equality constraints.
    ///
    /// ┌────────┐
    /// │Min(==x)│
    /// └────────┘
    pub const MIN_SIZE_EQ: Strength = Strength::MEDIUM.mul_f64(10.0);

    /// The strength to apply to Max equality constraints.
    ///
    /// ┌────────┐
    /// │Max(==x)│
    /// └────────┘
    pub const MAX_SIZE_EQ: Strength = Strength::MEDIUM.mul_f64(10.0);

    /// The strength to apply to Fill growing constraints.
    ///
    /// ┌─────────────────────┐
    /// │<=     Fill(x)     =>│
    /// └─────────────────────┘
    pub const FILL_GROW: Strength = Strength::MEDIUM;

    /// The strength to apply to growing constraints.
    ///
    /// ┌────────────┐
    /// │<= Min(x) =>│
    /// └────────────┘
    pub const GROW: Strength = Strength::MEDIUM.div_f64(10.0);

    /// The strength to apply to Spacer growing constraints.
    ///
    /// ┌       ┐
    ///  <= x =>
    /// └       ┘
    pub const SPACE_GROW: Strength = Strength::WEAK.mul_f64(10.0);

    /// The strength to apply to growing the size of all segments equally.
    ///
    /// ┌───────┐
    /// │<= x =>│
    /// └───────┘
    pub const ALL_SEGMENT_GROW: Strength = Strength::WEAK;
}
