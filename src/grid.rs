//! Structures and functions relating to crossword grids and entries.
//!
//! ----------------------------------------------------------------------------

/// A direction in the crossword grid space. Colloquial names for crossword
/// clue directions are used to ease reasoning and mental visuals, but in theory
/// these could be any direction on a 2D grid.
#[derive(Clone, Debug)]
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
    /// The maximum size of the grid in the across direction. If no size is
    /// given, the grid may be expanded as far in this direction as necessary to
    /// fit words in.
    size_across: Option<usize>,
    /// The maximum size of the grid in the down direction. If no size is given,
    /// the grid may be expanded as far in this direction as necessary to fit
    /// words in.
    size_down: Option<usize>,
    /// All of the valid entries in the grid. During grid _creation_, a separate
    /// list of "incomplete" entries may be tracked for future word placement
    /// candidates, but these entries _must_ always be valid.
    entries: Vec<Entry>,
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

/// Simple conversion from a tuple of two values to a grid coordinate. The
/// first value is the coordinate's across value, and the second the
/// coordinate's down value.
///
/// ```
/// let c = Coordinate::from((5usize, 1usize));
/// assert!(c.a == 5usize);
/// assert_eq!(c.d, 1usize);
/// ```
impl<T> From<(T, T)> for Coordinate where T: Into<usize> {
    fn from(value: (T, T)) -> Self {
        Self { a: value.0.into(), d: value.1.into() }
    }
}
