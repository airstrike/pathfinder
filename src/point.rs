// Largely based on the original `Point` from [`iced`].
//
// [`iced`]: https://github.com/iced-rs/iced
//
// Copyright 2019 Héctor Ramón, Iced contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
use crate::Vector;

use num_traits::{Float, Num};
use std::fmt;

/// A 2D point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Point<T = i32> {
    /// The X coordinate.
    pub x: T,

    /// The Y coordinate.
    pub y: T,
}

impl Point {
    /// The origin (i.e. a [`Point`] at (0, 0)).
    pub const ORIGIN: Self = Self::new(0, 0);
}

impl<T: Num> Point<T> {
    /// Creates a new [`Point`] with the given coordinates.
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Computes the distance to another [`Point`].
    pub fn distance(&self, to: Self) -> T
    where
        T: Float,
    {
        let a = self.x - to.x;
        let b = self.y - to.y;

        a.hypot(b)
    }
}

impl<T> From<[T; 2]> for Point<T>
where
    T: Num,
{
    fn from([x, y]: [T; 2]) -> Self {
        Point { x, y }
    }
}

impl<T> From<(T, T)> for Point<T>
where
    T: Num,
{
    fn from((x, y): (T, T)) -> Self {
        Self { x, y }
    }
}

impl<T> From<Point<T>> for [T; 2] {
    fn from(point: Point<T>) -> [T; 2] {
        [point.x, point.y]
    }
}

impl<T> std::ops::Add<Vector<T>> for Point<T>
where
    T: std::ops::Add<Output = T>,
{
    type Output = Self;

    fn add(self, vector: Vector<T>) -> Self {
        Self {
            x: self.x + vector.x,
            y: self.y + vector.y,
        }
    }
}

impl<T> std::ops::Sub<Vector<T>> for Point<T>
where
    T: std::ops::Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, vector: Vector<T>) -> Self {
        Self {
            x: self.x - vector.x,
            y: self.y - vector.y,
        }
    }
}

impl<T> std::ops::Sub<Point<T>> for Point<T>
where
    T: std::ops::Sub<Output = T>,
{
    type Output = Vector<T>;

    fn sub(self, point: Self) -> Vector<T> {
        Vector::new(self.x - point.x, self.y - point.y)
    }
}

impl<T> fmt::Display for Point<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Point {{ x: {}, y: {} }}", self.x, self.y)
    }
}

/// Converts a [`Point`] to an [`iced::Point`].
impl<T> From<Point<T>> for iced::Point<T> {
    fn from(point: Point<T>) -> iced::Point<T> {
        iced::Point {
            x: point.x,
            y: point.y,
        }
    }
}
