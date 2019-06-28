use generic_array::{ArrayLength, GenericArray};
// use std::ops::Deref;
use typenum::{Unsigned, U1, U2, U3, U4, U5, U6};

pub type Grid1 = Grid<U1>;
pub type Grid2 = Grid<U2>;
pub type Grid3 = Grid<U3>;
pub type Grid4 = Grid<U4>;
pub type Grid5 = Grid<U5>;
pub type Grid6 = Grid<U6>;

pub struct Grid<U: Unsigned> {
    grid: term_grid::Grid,
    _width: std::marker::PhantomData<U>,
}

impl<U: Unsigned> Default for Grid<U> {
    fn default() -> Self {
        Grid::new(term_grid::GridOptions {
            filling: term_grid::Filling::Spaces(1),
            // filling: term_grid::Filling::Text(String::from(" | ")),
            direction: term_grid::Direction::LeftToRight,
        })
    }
}

impl<U: Unsigned> Grid<U> {
    pub fn new(options: term_grid::GridOptions) -> Self {
        Grid {
            grid: term_grid::Grid::new(options),
            _width: std::marker::PhantomData,
        }
    }

    pub fn title(&mut self, title: &str) {
        for _ in 0..(U::to_usize()) {
            self.grid.add(term_grid::Cell::from(title));
        }
    }

    pub fn row<'a, 'b, R, T>(&'a mut self, row: R)
    where
        U: ArrayLength<T>,
        GenericArray<T, U>: From<R>,
        T: std::fmt::Display,
    {
        for s in GenericArray::from(row) {
            self.grid.add(term_grid::Cell::from(format!("{}", s)))
        }
    }

    pub fn display(&self) -> term_grid::Display {
        self.grid.fit_into_columns(U::to_usize())
    }
}
