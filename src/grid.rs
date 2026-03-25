//! Structures and functions relating to crossword grids and entries.
//!
//! ----------------------------------------------------------------------------

/// A direction in the crossword grid space. Colloquial names for crossword
/// clue directions are used to ease reasoning and mental visuals, but in theory
/// these could be any direction on a 2D grid.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GridDirection {
    /// The "across" direction. On paper, this is typically horizontal toward
    /// the right.
    Across,
    /// The "down" direction. On paper, this is typically vertical toward the
    /// bottom.
    Down,
}

/// A coordinate in the crossword grid.
///
/// Coordinates start in the upper-left of the grid.
#[derive(Clone, Copy, Debug, Default)]
pub struct Coordinate {
    /// The across coordinate (e.g. horizontal).
    a: usize,
    /// The down coordinate (e.g. vertical).
    d: usize,
}

/// A word entry in the crossword.
#[derive(Clone, Debug)]
pub struct Entry {
    /// The starting point of the entry in the crossword grid. The first element
    /// of the entry's text goes in this coordinate, with the remaining text
    /// following in the direction of this entry.
    coordinate: Coordinate,
    /// The entry's text. Note that this may be incomplete if the entry was
    /// created as a side-effect from the insertion of a valid entry.
    text: String,
    direction: GridDirection,
}

/// The crossword grid.
#[derive(Debug)]
pub struct Grid {
    /// The maximum coordinate of the grid in the across direction. If no size
    /// is given, the grid may be expanded as far in this direction as necessary
    /// to fit words in.
    size_across: Option<usize>,
    /// The maximum coordinate of the grid in the down direction. If no size is
    /// given, the grid may be expanded as far in this direction as necessary to
    /// fit words in.
    size_down: Option<usize>,
    // TODO: this is kind of a lie. We should keep track of the invalid entries
    //       here because the "add" method doesn't check validity, and may
    //       insert a word that completely invalidates the grid.
    /// All of the valid entries in the grid. During grid _creation_, a separate
    /// list of "incomplete" entries may be tracked for future word placement
    /// candidates, but these entries _must_ always be valid.
    entries: Vec<Entry>,
}

pub struct IncompleteGrid {
    grid: Grid,
    /// Entries in the grid that are side-effects of adding to the main entries
    /// list. These are possibly-incomplete or invalid entries that must be
    /// satisfied in order for the grid to be considered satisfied.
    entry_fragments: Vec<Entry>,
}

impl Coordinate {
    /// Check if this coordinate is between two other coordinates. If the two
    /// other coordinates do not create an across or down line (e.g. they are
    /// diagonal), this will check if this coordinate is within the bounding box
    /// described by the other two coordinates. Note that the comparison is
    /// inclusive of the bounds.
    ///
    /// The bounding coordinates do not need to be sorted/ordered in any way.
    #[inline(always)]
    fn is_within(self, bound1: Self, bound2: Self) -> bool {
        let min_across = std::cmp::min(bound1.a, bound2.a);
        let max_across = std::cmp::max(bound1.a, bound2.a);
        let min_down = std::cmp::min(bound1.d, bound2.d);
        let max_down = std::cmp::max(bound1.d, bound2.d);

        min_across <= self.a
            && self.a <= max_across
            && min_down <= self.d
            && self.d <= max_down
    }
}

impl Entry {
    /// Create a new entry. The text can be anything convertible to a `String`
    /// (e.g. it implements `ToString`). The coordinates may be anything
    /// convertible to a `Coordinate`. See documentation for
    /// [`Coordinate::from<(T, T)>`] for how a tuple of two values is
    /// interpreted.
    pub fn new<T: ToString, C: Into<Coordinate>>(
        text: T,
        coordinate: C,
        direction: GridDirection,
    ) -> Self {
        Self {
            coordinate: coordinate.into(),
            text: text.to_string(),
            direction,
        }
    }
}

/// A collection of methods for getting coordinate bounds of entries. These are
/// useful when determining grid validity for entries or defining grid bounds
/// from a set of entries.
///
/// ```
/// let e = Entry::new("twiddle", (3, 5), GridDirection::Across);
/// assert_eq!(e.max_across(), 10);
/// assert_eq!(e.max_down(), 5);
/// assert_eq!(e.min_across(), 3);
/// assert_eq!(e.min_down(), 5);
///
/// let e = Entry::new("growl", (1, 2), GridDirection::Down);
/// assert_eq!(e.max_across(), 1);
/// assert_eq!(e.max_down(), 7);
/// assert_eq!(e.min_across(), 1);
/// assert_eq!(e.min_down(), 2);
/// ```
impl Entry {
    /// Return the maximum across coordinate used by this entry. If the entry is
    /// in the across direction, this will be its across coordinate plus its
    /// length. Otherwise, it is just the across coordinate.
    pub fn max_across(&self) -> usize {
        match self.direction {
            GridDirection::Across => self.coordinate.a + self.text.len() - 1,
            GridDirection::Down => self.coordinate.a,
        }
    }

    /// Return the maximum down coordinate used by this entry. If the entry is
    /// in the down direction, this will be its down coordinate plus its
    /// length. Otherwise, it is just the down coordinate.
    pub fn max_down(&self) -> usize {
        match self.direction {
            GridDirection::Across => self.coordinate.d,
            GridDirection::Down => self.coordinate.d + self.text.len() - 1,
        }
    }

    /// Return the minimum across coordinate used by this entry. This is simply
    /// the same as the across coordinate in the entry's [`Entry::coordinate`]
    /// field.
    pub fn min_across(&self) -> usize {
        return self.coordinate.a;
    }

    /// Return the minimum down coordinate used by this entry. This is simply
    /// the same as the down coordinate in the entry's [`Entry::coordinate`]
    /// field.
    pub fn min_down(&self) -> usize {
        return self.coordinate.d;
    }
}

/// Simple conversion from a tuple of two values to a grid coordinate. The
/// first value is the coordinate's across value, and the second the
/// coordinate's down value.
///
/// ```
/// let c = Coordinate::from((5usize, 1usize));
/// assert!(c.a == 5usize);
/// assert_eq!(c.d, 1usize);
/// ```
impl<T> From<(T, T)> for Coordinate
where
    T: Into<usize>,
{
    fn from(value: (T, T)) -> Self {
        Self { a: value.0.into(), d: value.1.into() }
    }
}

/// Create a grid from a list of entries. The grid size will be determined by
/// the maximum coordinates used by the entries.
///
/// ```
/// // Example puzzle:
/// //   0 1 2 3 4 5
/// // 0       d
/// // 1 a c r o s s
/// // 2       w
/// // 3       n
/// let e = vec![
///     Entry::new("across", (0usize, 1usize), GridDirection::Across),
///     Entry::new("down", (3usize, 0usize), GridDirection::Down),
/// ];
/// let g = Grid::from(e.as_slice());
/// assert_eq!(g.size_across, Some(5usize));
/// assert_eq!(g.size_down, Some(3usize));
/// ```
impl From<&[Entry]> for Grid {
    fn from(value: &[Entry]) -> Self {
        let size_across = value.into_iter().map(Entry::max_across).max();
        let size_down = value.into_iter().map(Entry::max_down).max();
        Self { size_across, size_down, entries: value.into() }
    }
}
