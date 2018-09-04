use ndarray::prelude::*;
use ndarray::Data;
use noisy_float::types::{N32, N64};

/// A number type that can have not-a-number values.
pub trait MaybeNan: Sized {
    /// A type that is guaranteed not to be a NaN value.
    type NotNan;

    /// Returns `true` if the value is a NaN value.
    fn is_nan(&self) -> bool;

    /// Tries to convert the value to `NotNan`.
    ///
    /// Returns `None` if the value is a NaN value.
    fn try_as_not_nan(&self) -> Option<&Self::NotNan>;

    /// Converts the value.
    ///
    /// If the value is `None`, a NaN value is returned.
    fn from_not_nan(Self::NotNan) -> Self;

    /// Converts the value.
    ///
    /// If the value is `None`, a NaN value is returned.
    fn from_not_nan_opt(Option<Self::NotNan>) -> Self;

    /// Converts the value.
    ///
    /// If the value is `None`, a NaN value is returned.
    fn from_not_nan_ref_opt(Option<&Self::NotNan>) -> &Self;

    /// Returns a view with the NaN values removed.
    ///
    /// This modifies the input view by moving elements as necessary. The final
    /// order of the elements is unspecified. However, this method is
    /// idempotent, and given the same input data, the result is always ordered
    /// the same way.
    fn remove_nan_mut(ArrayViewMut1<Self>) -> ArrayViewMut1<Self::NotNan>;
}

/// Returns a view with the NaN values removed.
///
/// This modifies the input view by moving elements as necessary.
fn remove_nan_mut<A: MaybeNan>(mut view: ArrayViewMut1<A>) -> ArrayViewMut1<A> {
    if view.len() == 0 {
        return view.slice_move(s![..0]);
    }
    let mut i = 0;
    let mut j = view.len() - 1;
    loop {
        // At this point, `i == 0 || !view[i-1].is_nan()`
        // and `j == view.len() - 1 || view[j+1].is_nan()`.
        while i <= j && !view[i].is_nan() {
            i += 1;
        }
        // At this point, `view[i].is_nan() || i == j + 1`.
        while j > i && view[j].is_nan() {
            j -= 1;
        }
        // At this point, `!view[j].is_nan() || j == i`.
        if i >= j {
            return view.slice_move(s![..i]);
        } else {
            view.swap(i, j);
            i += 1;
            j -= 1;
        }
    }
}

macro_rules! impl_maybenan_for_fxx {
    ($fxx:ident, $Nxx:ident) => {
        impl MaybeNan for $fxx {
            type NotNan = $Nxx;

            fn is_nan(&self) -> bool {
                $fxx::is_nan(*self)
            }

            fn try_as_not_nan(&self) -> Option<&$Nxx> {
                if self.is_nan() {
                    None
                } else {
                    // This is safe because `$Nxx` is a thin `repr(C)` wrapper
                    // around `$fxx`, and we have just checked that `self` is
                    // not a NaN value.
                    Some(unsafe { &*(self as *const $fxx as *const $Nxx) })
                }
            }

            fn from_not_nan(value: $Nxx) -> $fxx {
                value.raw()
            }

            fn from_not_nan_opt(value: Option<$Nxx>) -> $fxx {
                match value {
                    None => ::std::$fxx::NAN,
                    Some(num) => num.raw(),
                }
            }

            fn from_not_nan_ref_opt(value: Option<&$Nxx>) -> &$fxx {
                match value {
                    None => &::std::$fxx::NAN,
                    // This is safe because `$Nxx` is a thin `repr(C)` wrapper
                    // around `$fxx`.
                    Some(num) => unsafe { &*(num as *const $Nxx as *const $fxx) },
                }
            }

            fn remove_nan_mut(view: ArrayViewMut1<$fxx>) -> ArrayViewMut1<$Nxx> {
                let not_nan = remove_nan_mut(view);
                // This is safe because `remove_nan_mut` has removed the NaN
                // values, and `$Nxx` is a thin wrapper around `$fxx`.
                unsafe {
                    ArrayViewMut1::from_shape_ptr(not_nan.dim(), not_nan.as_ptr() as *mut $Nxx)
                }
            }
        }
    };
}
impl_maybenan_for_fxx!(f32, N32);
impl_maybenan_for_fxx!(f64, N64);

macro_rules! impl_maybenan_for_opt_never_nan {
    ($ty:ty) => {
        impl MaybeNan for Option<$ty> {
            type NotNan = NotNone<$ty>;

            fn is_nan(&self) -> bool {
                self.is_none()
            }

            fn try_as_not_nan(&self) -> Option<&NotNone<$ty>> {
                if self.is_none() {
                    None
                } else {
                    // This is safe because we have checked for the `None`
                    // case, and `NotNone<$ty>` is a thin wrapper around `Option<$ty>`.
                    Some(unsafe { &*(self as *const Option<$ty> as *const NotNone<$ty>) })
                }
            }

            fn from_not_nan(value: NotNone<$ty>) -> Option<$ty> {
                value.into_inner()
            }

            fn from_not_nan_opt(value: Option<NotNone<$ty>>) -> Option<$ty> {
                value.and_then(|v| v.into_inner())
            }

            fn from_not_nan_ref_opt(value: Option<&NotNone<$ty>>) -> &Option<$ty> {
                match value {
                    None => &None,
                    // This is safe because `NotNone<$ty>` is a thin wrapper around
                    // `Option<$ty>`.
                    Some(num) => unsafe { &*(num as *const NotNone<$ty> as *const Option<$ty>) },
                }
            }

            fn remove_nan_mut(view: ArrayViewMut1<Self>) -> ArrayViewMut1<Self::NotNan> {
                let not_nan = remove_nan_mut(view);
                // This is safe because `remove_nan_mut` has removed the `None`
                // values, and `NotNone<$ty>` is a thin wrapper around `Option<$ty>`.
                unsafe {
                    ArrayViewMut1::from_shape_ptr(
                        not_nan.dim(),
                        not_nan.as_ptr() as *mut NotNone<$ty>,
                    )
                }
            }
        }
    };
}
impl_maybenan_for_opt_never_nan!(u8);
impl_maybenan_for_opt_never_nan!(u16);
impl_maybenan_for_opt_never_nan!(u32);
impl_maybenan_for_opt_never_nan!(u64);
impl_maybenan_for_opt_never_nan!(u128);
impl_maybenan_for_opt_never_nan!(i8);
impl_maybenan_for_opt_never_nan!(i16);
impl_maybenan_for_opt_never_nan!(i32);
impl_maybenan_for_opt_never_nan!(i64);
impl_maybenan_for_opt_never_nan!(i128);
impl_maybenan_for_opt_never_nan!(N32);
impl_maybenan_for_opt_never_nan!(N64);

/// A thin wrapper around `Option` that guarantees that the value is not
/// `None`.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NotNone<T>(Option<T>);

impl<T> NotNone<T> {
    /// Returns the underling option.
    pub fn into_inner(self) -> Option<T> {
        self.0
    }

    /// Moves the value out of the inner option.
    ///
    /// This method is guaranteed not to panic.
    pub fn unwrap(self) -> T {
        match self.0 {
            Some(inner) => inner,
            None => unsafe { ::std::hint::unreachable_unchecked() },
        }
    }
}

/// Extension trait for `ArrayBase` providing NaN-related functionality.
pub trait MaybeNanExt<A, S, D>
where
    A: MaybeNan,
    S: Data<Elem = A>,
    D: Dimension,
{
    /// Traverse the non-NaN array elements and apply a fold, returning the
    /// resulting value.
    ///
    /// Elements are visited in arbitrary order.
    fn fold_skipnan<'a, F, B>(&'a self, init: B, f: F) -> B
    where
        A: 'a,
        F: FnMut(B, &'a A::NotNan) -> B;
}

impl<A, S, D> MaybeNanExt<A, S, D> for ArrayBase<S, D>
where
    A: MaybeNan,
    S: Data<Elem = A>,
    D: Dimension,
{
    fn fold_skipnan<'a, F, B>(&'a self, init: B, mut f: F) -> B
    where
        A: 'a,
        F: FnMut(B, &'a A::NotNan) -> B,
    {
        self.fold(init, |acc, elem| {
            if let Some(not_nan) = elem.try_as_not_nan() {
                f(acc, not_nan)
            } else {
                acc
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    quickcheck! {
        fn remove_nan_mut_idempotent(is_nan: Vec<bool>) -> bool {
            let mut values: Vec<_> = is_nan
                .into_iter()
                .map(|is_nan| if is_nan { None } else { Some(1) })
                .collect();
            let view = ArrayViewMut1::from_shape(values.len(), &mut values).unwrap();
            let removed = remove_nan_mut(view);
            removed == remove_nan_mut(removed.to_owned().view_mut())
        }

        fn remove_nan_mut_only_nan_remaining(is_nan: Vec<bool>) -> bool {
            let mut values: Vec<_> = is_nan
                .into_iter()
                .map(|is_nan| if is_nan { None } else { Some(1) })
                .collect();
            let view = ArrayViewMut1::from_shape(values.len(), &mut values).unwrap();
            remove_nan_mut(view).iter().all(|elem| !elem.is_nan())
        }

        fn remove_nan_mut_keep_all_non_nan(is_nan: Vec<bool>) -> bool {
            let non_nan_count = is_nan.iter().filter(|&&is_nan| !is_nan).count();
            let mut values: Vec<_> = is_nan
                .into_iter()
                .map(|is_nan| if is_nan { None } else { Some(1) })
                .collect();
            let view = ArrayViewMut1::from_shape(values.len(), &mut values).unwrap();
            remove_nan_mut(view).len() == non_nan_count
        }
    }
}

mod impl_not_none;