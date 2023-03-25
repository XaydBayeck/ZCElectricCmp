pub trait Gen<Dev> {
    fn get(self) -> Dev;
}
