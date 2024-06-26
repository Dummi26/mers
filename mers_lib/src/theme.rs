pub trait ThemeGen {
    type C;
    type T;
    fn color(&self, text: &str, color: Self::C, t: &mut Self::T);
    fn nocolor(&self, text: &str, t: &mut Self::T);
}
pub trait ThemeTo: ThemeGen<T = String> {
    fn color_to(&self, text: &str, color: <Self as ThemeGen>::C) -> <Self as ThemeGen>::T;
}
impl<T: ThemeGen<T = String> + ?Sized> ThemeTo for T {
    fn color_to(&self, text: &str, color: <T as ThemeGen>::C) -> String {
        let mut t = String::new();
        self.color(text, color, &mut t);
        t
    }
}
