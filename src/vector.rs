// Largely based on the original `Vector` from [`iced`].
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

/// A 2D vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vector<T = f32> {
    /// The X component of the [`Vector`]
    pub x: T,

    /// The Y component of the [`Vector`]
    pub y: T,
}

impl<T> Vector<T> {
    /// Creates a new [`Vector`] with the given components.
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl Vector {
    /// The zero [`Vector`].
    pub const ZERO: Self = Self::new(0.0, 0.0);
}

impl<T> std::ops::Neg for Vector<T>
where
    T: std::ops::Neg<Output = T>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl<T> std::ops::Add for Vector<T>
where
    T: std::ops::Add<Output = T>,
{
    type Output = Self;

    fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y)
    }
}

impl<T> std::ops::Sub for Vector<T>
where
    T: std::ops::Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y)
    }
}

impl<T> std::ops::Mul<T> for Vector<T>
where
    T: std::ops::Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, scale: T) -> Self {
        Self::new(self.x * scale, self.y * scale)
    }
}

impl<T> Default for Vector<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            x: T::default(),
            y: T::default(),
        }
    }
}

impl<T> From<[T; 2]> for Vector<T> {
    fn from([x, y]: [T; 2]) -> Self {
        Self::new(x, y)
    }
}

impl<T> From<Vector<T>> for [T; 2]
where
    T: Copy,
{
    fn from(other: Vector<T>) -> Self {
        [other.x, other.y]
    }
}

impl<T> From<Vector<T>> for iced::Vector<T> {
    fn from(vector: Vector<T>) -> Self {
        Self::new(vector.x, vector.y)
    }
}
