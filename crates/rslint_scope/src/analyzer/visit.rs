use crate::datalog::DatalogBuilder;

pub trait Visit<'ddlog, T> {
    type Output;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, item: T) -> Self::Output;
}

impl<'ddlog, T, V> Visit<'ddlog, Option<T>> for V
where
    V: Visit<'ddlog, T>,
{
    type Output = Option<<V as Visit<'ddlog, T>>::Output>;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, option: Option<T>) -> Self::Output {
        option.map(|val| self.visit(scope, val))
    }
}
