//! Structures and functions relating to crossword grids and entries.
//!
//! ----------------------------------------------------------------------------

/// A direction in the crossword grid space. Colloquial names for crossword
/// clue directions are used to ease reasoning and mental visuals, but in theory
/// these could be any direction on a 2D grid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    fn is_within(
        self,
        bound1: impl Into<Self>,
        bound2: impl Into<Self>,
    ) -> bool {
        let c1 = bound1.into();
        let c2 = bound2.into();

        let min_across = std::cmp::min(c1.a, c2.a);
        let max_across = std::cmp::max(c1.a, c2.a);
        let min_down = std::cmp::min(c1.d, c2.d);
        let max_down = std::cmp::max(c1.d, c2.d);

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

/// Methods pertaining to comparisons between two entries.
impl Entry {
    #[inline(always)]
    /// Return whether this entry is parallel (in the same direction as) another
    /// entry.
    pub fn is_parallel_to(&self, other: &Self) -> bool {
        self.direction == other.direction
    }

    /// Check whether this entry overlaps with another entry. This is distinct
    /// from an intersection in that entries must be parallel for them to be
    /// considered overlapping.
    ///
    /// For example, the grid below shows the words overlapping:
    ///
    /// ```
    /// //   0 1
    /// // 0   i
    /// // 1   n
    /// // 2   tt
    /// // 3   oo
    /// // 4    p
    /// let e1 = Entry::new("into", (1usize, 0usize), GridDirection::Down);
    /// let e2 = Entry::new("top", (1usize, 2usize), GridDirection::Down);
    /// assert!(e1.overlaps(e2));
    /// ```
    pub fn overlaps(&self, other: &Self) -> bool {
        let ss = self.start_coordinate();
        let se = self.end_coordinate();
        let os = other.start_coordinate();
        let oe = other.end_coordinate();

        if self.is_parallel_to(other) {
            // If either the start or the end coordinate of the other entry lies
            // between (inclusive) this entry's start and end, and they are
            // collinear, then there is overlap.
            let is_within = os.is_within(ss, se) || oe.is_within(ss, se);
            let is_same_level = match self.direction {
                GridDirection::Across => {
                    // This has to be true since they are marked as parallel.
                    debug_assert_eq!(other.direction, GridDirection::Across);

                    // Overlapping if the coordinates are on the same level AND
                    // either the start or the end coordinate of the other.
                    self.coordinate.d == other.coordinate.d
                },
                GridDirection::Down => {
                    // This has to be true since they are marked as parallel.
                    debug_assert_eq!(other.direction, GridDirection::Down);
                    self.coordinate.a == other.coordinate.a
                },
            };
            is_within && is_same_level
        } else {
            false
        }
    }

    /// Check whether this entry intersects with another entry. This is distinct
    /// from an overlap in that entries must be perpendicular for them to be
    /// considered intersecting.
    ///
    /// For example, the grid below shows the words intersecting:
    ///
    /// ```
    /// //   0 1 2 3
    /// // 0   e
    /// // 1 c r a b
    /// // 2   a
    /// let e1 = Entry::new("era", (1usize, 0usize), GridDirection::Down);
    /// let e2 = Entry::new("crab", (0usize, 1usize), GridDirection::Across);
    /// assert!(e1.intersects(e2));
    /// ```
    pub fn intersects(&self, other: &Self) -> bool {
        let ss = self.start_coordinate();
        let se = self.end_coordinate();
        let os = other.start_coordinate();
        let oe = other.end_coordinate();

        if !self.is_parallel_to(other) {
            match (self.direction, other.direction) {
                (GridDirection::Across, GridDirection::Across)
                | (GridDirection::Down, GridDirection::Down) => false,
                (GridDirection::Across, GridDirection::Down) => {
                    (os.d <= ss.d && ss.d <= oe.d)
                        && (ss.a <= os.a && os.a <= se.a)
                },
                (GridDirection::Down, GridDirection::Across) => {
                    (os.a <= ss.a && ss.a <= oe.a)
                        && (ss.d <= os.d && os.d <= se.d)
                },
            }
        } else {
            false
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

    /// The coordinate at which the entry starts.
    pub fn start_coordinate(&self) -> Coordinate {
        return self.coordinate;
    }

    /// The coordinate at which the entry ends.
    pub fn end_coordinate(&self) -> Coordinate {
        match self.direction {
            GridDirection::Across => Coordinate {
                a: self.coordinate.a + self.text.len() - 1,
                d: self.coordinate.d,
            },
            GridDirection::Down => Coordinate {
                a: self.coordinate.a,
                d: self.coordinate.d + self.text.len() - 1,
            },
        }
    }
}

/// Simple conversion from a grid entry into a grid coordinate so it can be used
/// with coordinate-comparing methods/functions.
impl Into<Coordinate> for Entry {
    fn into(self) -> Coordinate {
        self.coordinate
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

#[test]
fn test_coordinate_is_within() {
    let tests: [[(usize, usize); _]; _] = [
        //   0 1 2 3
        // 0 +-----x
        // 1 | 1 2 |
        // 2 | 3 4 |
        // 3 x-----+
        [(0, 3), (3, 0), (1, 1)],
        [(0, 3), (3, 0), (2, 1)],
        [(0, 3), (3, 0), (1, 2)],
        [(0, 3), (3, 0), (2, 2)],

        //   0 1
        // 0 1-x
        // 1 x-2
        [(1, 0), (0, 1), (0, 0)],
        [(1, 0), (0, 1), (1, 1)],
        // Also test the bounds themselves
        [(1, 0), (0, 1), (1, 0)],
        [(1, 0), (0, 1), (0, 1)],
    ];

    for test in tests {
        let [c1, c2, c3] = test.map(Coordinate::from);
        assert!(c3.is_within(c1, c2));
        assert!(c3.is_within(c2, c1));
    }
}

#[test]
fn test_entries_overlap() {
    let tests = [
        //     0 1 2 3 4 5
        //   0 c u t
        //       u t t e r
        (
            Entry::new("cut", (0usize, 0usize), GridDirection::Across),
            Entry::new("utter", (1usize, 0usize), GridDirection::Across),
        ),
        //     0 1 2 3 4 5 6 7
        //   0 y o u r s
        //         u n r e a l
        (
            Entry::new("yours", (0usize, 0usize), GridDirection::Across),
            Entry::new("unreal", (2usize, 0usize), GridDirection::Across),
        ),
    ];

    for test in tests {
        let (e1, e2) = test;
        assert!(e1.overlaps(&e2), "Entries should overlap: {e1:?} -> {e2:?}");

        // Intersections between entries are symmetric.
        assert!(
            e2.overlaps(&e1),
            "Entries should overlap symmetrically: {e1:?} -> {e2:?}"
        );
    }
}

#[test]
fn test_entries_do_not_overlap() {
    let tests = [
        //     0 1 2 3 4 5
        //   0 c u t
        //           u t t e r
        (
            Entry::new("cut", (0usize, 0usize), GridDirection::Across),
            Entry::new("utter", (3usize, 0usize), GridDirection::Across),
        ),
        //     0 1 2 3 4 5 6 7
        //   0 y o u r s
        //               u n r e a l
        (
            Entry::new("yours", (0usize, 0usize), GridDirection::Across),
            Entry::new("unreal", (5usize, 0usize), GridDirection::Across),
        ),
    ];

    for test in tests {
        let (e1, e2) = test;
        assert!(
            !e1.overlaps(&e2),
            "Entries should not overlap: {e1:?} -> {e2:?}"
        );

        // Intersections between entries are symmetric.
        assert!(
            !e2.overlaps(&e1),
            "Entries should not overlap symmetrically: {e1:?} -> {e2:?}"
        );
    }
}

#[test]
fn test_entries_intersect() {
    let tests = [
        //     0 1 2
        //   0 o u t
        //   1   n
        //   2   d
        //   3   o
        (
            Entry::new("out", (0usize, 0usize), GridDirection::Across),
            Entry::new("undo", (1usize, 0usize), GridDirection::Down),
        ),
        //     0 1 2 3 4 5 6
        //   0 t u r m o i l
        //   1       o
        //   2       u
        //   3       n
        //   4       t
        (
            Entry::new("turnmoil", (0usize, 0usize), GridDirection::Across),
            Entry::new("mount", (3usize, 0usize), GridDirection::Down),
        ),
    ];

    for test in tests {
        let (e1, e2) = test;
        assert!(
            e1.intersects(&e2),
            "Entries should intersect: {e1:?} -> {e2:?}"
        );

        // Intersections between entries are symmetric.
        assert!(
            e2.intersects(&e1),
            "Entries should intersect: {e1:?} -> {e2:?}"
        );
    }
}
