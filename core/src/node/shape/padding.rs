use crate::RealValue;

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Padding {
    pub top: RealValue,
    pub left: RealValue,
    pub right: RealValue,
    pub bottom: RealValue,
}

impl Padding {
    pub fn top_and_bottom(&self) -> RealValue {
        self.top + self.bottom
    }

    pub fn left_and_right(&self) -> RealValue {
        self.left + self.right
    }
}

impl<T: Into<RealValue>> From<T> for Padding {
    fn from(padding: T) -> Self {
        let padding = padding.into();
        Self {
            top: padding,
            left: padding,
            right: padding,
            bottom: padding,
        }
    }
}
