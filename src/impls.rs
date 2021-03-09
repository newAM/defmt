use crate as defmt;
use defmt_macros::internp;

use crate::{Format, Formatter, Str};

impl Format for i8 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=i8}");
            fmt.inner.u8(&t);
        }
        fmt.inner.u8(&(*self as u8));
    }
}

impl Format for i16 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=i16}");
            fmt.inner.u8(&t);
        }
        fmt.inner.u16(&(*self as u16))
    }
}

impl Format for i32 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=i32}");
            fmt.inner.u8(&t);
        }
        fmt.inner.i32(self);
    }
}

impl Format for i64 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=i64}");
            fmt.inner.u8(&t);
        }
        fmt.inner.i64(self);
    }
}

impl Format for i128 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=i128}");
            fmt.inner.u8(&t);
        }
        fmt.inner.i128(self);
    }
}

impl Format for isize {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=isize}");
            fmt.inner.u8(&t);
        }
        fmt.inner.isize(self);
    }
}

impl Format for u8 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=u8}");
            fmt.inner.u8(&t);
        }
        fmt.inner.u8(self)
    }
}

impl Format for u16 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=u16}");
            fmt.inner.u8(&t);
        }
        fmt.inner.u16(self);
    }
}

impl Format for u32 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=u32}");
            fmt.inner.u8(&t);
        }
        fmt.inner.u32(self);
    }
}

impl Format for u64 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=u64}");
            fmt.inner.u8(&t);
        }
        fmt.inner.u64(self);
    }
}

impl Format for u128 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=u128}");
            fmt.inner.u8(&t);
        }
        fmt.inner.u128(self);
    }
}

impl Format for usize {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=usize}");
            fmt.inner.u8(&t);
        }
        fmt.inner.usize(self);
    }
}

impl Format for f32 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=f32}");
            fmt.inner.u8(&t);
        }
        fmt.inner.f32(self);
    }
}

impl Format for f64 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=f64}");
            fmt.inner.u8(&t);
        }
        fmt.inner.f64(self);
    }
}

impl Format for str {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = str_tag();
            fmt.inner.u8(&t);
        }
        fmt.inner.str(self);
    }
}

pub(crate) fn str_tag() -> u8 {
    internp!("{=str}")
}

impl Format for Str {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=istr}");
            fmt.inner.u8(&t);
        }
        fmt.inner.istr(self);
    }
}

impl Format for char {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=char}");
            fmt.inner.u8(&t);
        }
        fmt.inner.u32(&(*self as u32));
    }
}

impl<T> Format for [T]
where
    T: Format,
{
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=[?]}");
            fmt.inner.u8(&t);
        }
        fmt.inner.fmt_slice(self)
    }
}

impl<T> Format for &'_ T
where
    T: Format + ?Sized,
{
    fn format(&self, fmt: Formatter) {
        T::format(self, fmt)
    }
}

impl<T> Format for &'_ mut T
where
    T: Format + ?Sized,
{
    fn format(&self, fmt: Formatter) {
        T::format(self, fmt)
    }
}

impl Format for bool {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=bool}");
            fmt.inner.u8(&t);
        }
        fmt.inner.bool(self);
    }
}

impl<T, const N: usize> Format for [T; N]
where
    T: Format,
{
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=[?;0]}");
            fmt.inner.u8(&t);
        }
        fmt.inner.fmt_array(self);
    }
}

impl<T> Format for Option<T>
where
    T: Format,
{
    fn format(&self, f: Formatter) {
        if f.inner.needs_tag() {
            let t = internp!("None|Some({=?})");
            f.inner.u8(&t);
        }
        match self {
            None => f.inner.u8(&0),
            Some(x) => {
                f.inner.u8(&1);
                f.inner.with_tag(|f| x.format(f))
            }
        }
    }
}

impl<T, E> Format for Result<T, E>
where
    T: Format,
    E: Format,
{
    fn format(&self, f: Formatter) {
        if f.inner.needs_tag() {
            let t = internp!("Err({=?})|Ok({=?})");
            f.inner.u8(&t);
        }
        match self {
            Err(e) => {
                f.inner.u8(&0);
                f.inner.with_tag(|f| e.format(f))
            }
            Ok(x) => {
                f.inner.u8(&1);
                f.inner.with_tag(|f| x.format(f))
            }
        }
    }
}

impl Format for () {
    fn format(&self, f: Formatter) {
        if f.inner.needs_tag() {
            let t = internp!("()");
            f.inner.u8(&t);
        }
    }
}

impl<T> Format for core::marker::PhantomData<T> {
    fn format(&self, f: Formatter) {
        if f.inner.needs_tag() {
            let t = internp!("PhantomData");
            f.inner.u8(&t);
        }
    }
}

macro_rules! tuple {
    ( $format:expr, ($($name:ident),+) ) => (
        impl<$($name:Format),+> Format for ($($name,)+) where last_type!($($name,)+): ?Sized {
            #[allow(non_snake_case, unused_assignments)]
            fn format(&self, f: Formatter) {
                if f.inner.needs_tag() {
                    let t = internp!($format);
                    f.inner.u8(&t);
                }

                let ($(ref $name,)+) = *self;
                $(
                    let formatter = Formatter { inner: f.inner };
                    $name.format(formatter);
                )+
            }
        }
    )
}

macro_rules! last_type {
    ($a:ident,) => { $a };
    ($a:ident, $($rest_a:ident,)+) => { last_type!($($rest_a,)+) };
}

tuple! { "({=?})", (T0) }
tuple! { "({=?}, {=?})", (T0, T1) }
tuple! { "({=?}, {=?}, {=?})", (T0, T1, T2) }
tuple! { "({=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3) }
tuple! { "({=?}, {=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3, T4) }
tuple! { "({=?}, {=?}, {=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3, T4, T5) }
tuple! { "({=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3, T4, T5, T6) }
tuple! { "({=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3, T4, T5, T6, T7) }
tuple! { "({=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3, T4, T5, T6, T7, T8) }
tuple! { "({=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9) }
tuple! { "({=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10) }
tuple! { "({=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11) }

#[cfg(feature = "alloc")]
mod if_alloc {
    use crate::{Format, Formatter};

    impl<T> Format for alloc::boxed::Box<T>
    where
        T: ?Sized + Format,
    {
        fn format(&self, f: Formatter) {
            T::format(&*self, f)
        }
    }

    impl<T> Format for alloc::rc::Rc<T>
    where
        T: ?Sized + Format,
    {
        fn format(&self, f: Formatter) {
            T::format(&*self, f)
        }
    }

    #[cfg(not(no_cas))]
    impl<T> Format for alloc::sync::Arc<T>
    where
        T: ?Sized + Format,
    {
        fn format(&self, f: Formatter) {
            T::format(&*self, f)
        }
    }

    impl<T> Format for alloc::vec::Vec<T>
    where
        T: Format,
    {
        fn format(&self, f: Formatter) {
            self.as_slice().format(f)
        }
    }

    impl Format for alloc::string::String {
        fn format(&self, f: Formatter) {
            self.as_str().format(f)
        }
    }
}

impl Format for core::convert::Infallible {
    #[inline]
    fn format(&self, _: Formatter) {
        // type cannot be instantiated so nothing to do here
        match *self {}
    }
}

impl Format for core::time::Duration {
    fn format(&self, fmt: Formatter) {
        crate::write!(
            fmt,
            "Duration {{ secs: {=u64}, nanos: {=u32} }}",
            self.as_secs(),
            self.subsec_nanos(),
        )
    }
}
