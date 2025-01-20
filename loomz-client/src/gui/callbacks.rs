use super::components::GuiLabelCallbackValues;

pub(super) type RawCallbackValue = u64;
pub trait IntoGuiCallback {
    fn into_u64(self) -> RawCallbackValue;
    fn from_u64(value: RawCallbackValue) -> Self;
}

impl<T: Into<u64>+From<RawCallbackValue>> IntoGuiCallback for T {
    fn into_u64(self) -> RawCallbackValue { self.into() }
    fn from_u64(value: RawCallbackValue) -> Self { Self::from(value) }
}

#[derive(Copy, Clone)]
pub(super) enum GuiComponentCallbacksValue {
    None,
    Label(GuiLabelCallbackValues)
}

impl GuiComponentCallbacksValue {

    pub fn take(&mut self) -> Self {
        let mut other = Self::None;
        ::std::mem::swap(self, &mut other);
        other
    }

}
