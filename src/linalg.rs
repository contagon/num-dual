use crate::*;
use ndarray::*;
use ndarray_linalg::convert::replicate;
use ndarray_linalg::error::Result;
use ndarray_linalg::*;

pub trait SolveDual<A: Copy> {
    /// Solves a system of linear equations `A * x = b` where `A` is `self`, `b`
    /// is the argument, and `x` is the successful result.
    fn solve<S: Data<Elem = A>>(&self, b: &ArrayBase<S, Ix1>) -> Result<Array1<A>> {
        let mut b = replicate(b);
        self.solve_inplace(&mut b)?;
        Ok(b)
    }
    /// Solves a system of linear equations `A * x = b` where `A` is `self`, `b`
    /// is the argument, and `x` is the successful result.
    fn solve_into<S: DataMut<Elem = A>>(
        &self,
        mut b: ArrayBase<S, Ix1>,
    ) -> Result<ArrayBase<S, Ix1>> {
        self.solve_inplace(&mut b)?;
        Ok(b)
    }
    /// Solves a system of linear equations `A * x = b` where `A` is `self`, `b`
    /// is the argument, and `x` is the successful result.
    fn solve_inplace<'a, S: DataMut<Elem = A>>(
        &self,
        b: &'a mut ArrayBase<S, Ix1>,
    ) -> Result<&'a mut ArrayBase<S, Ix1>>;
}

impl<S: Data<Elem = f64>> SolveDual<f64> for ArrayBase<S, Ix2> {
    /// Solves a system of linear equations `A * x = b` where `A` is `self`, `b`
    /// is the argument, and `x` is the successful result.
    /// ```
    /// # use num_hyperdual::linalg::SolveDual;
    /// # use ndarray::{arr1, arr2};
    /// let a = arr2(&[[1.0, 3.0],
    ///                [5.0, 7.0]]);
    /// let b = arr1(&[10.0, 26.0]);
    /// let x = a.solve_into(b).unwrap();
    /// assert_eq!(x, arr1(&[1.0, 3.0]));
    /// ```
    fn solve_inplace<'a, Sb: DataMut<Elem = f64>>(
        &self,
        b: &'a mut ArrayBase<Sb, Ix1>,
    ) -> Result<&'a mut ArrayBase<Sb, Ix1>> {
        <Self as Solve<f64>>::solve_inplace(self, b)
    }
}

impl<S: Data<Elem = Dual64>> SolveDual<Dual64> for ArrayBase<S, Ix2> {
    /// Solves a system of linear equations `A * x = b` where `A` is `self`, `b`
    /// is the argument, and `x` is the successful result.
    /// ```
    /// # use num_hyperdual::Dual64;
    /// # use num_hyperdual::linalg::SolveDual;
    /// # use ndarray::{arr1, arr2};
    /// let a = arr2(&[[Dual64::new(1.0, 2.0), Dual64::new(3.0, 4.0)],
    ///                [Dual64::new(5.0, 6.0), Dual64::new(7.0, 8.0)]]);
    /// let b = arr1(&[Dual64::new(10.0, 28.0), Dual64::new(26.0, 68.0)]);
    /// let x = a.solve_into(b).unwrap();
    /// assert_eq!(x, arr1(&[Dual64::new(1.0, 2.0), Dual64::new(3.0, 4.0)]));
    /// ```
    fn solve_inplace<'a, Sb: DataMut<Elem = Dual64>>(
        &self,
        b: &'a mut ArrayBase<Sb, Ix1>,
    ) -> Result<&'a mut ArrayBase<Sb, Ix1>> {
        let f = self.map(Dual64::re).factorize_into()?;
        let dx0 = f.solve_into(b.map(Dual64::re))?;
        let dx1 = f.solve_into(b.mapv(|b| b.eps) - self.mapv(|s| s.eps).dot(&dx0))?;
        Zip::from(&dx0)
            .and(&dx1)
            .and(&mut *b)
            .apply(|&dx0, &dx1, b| *b = Dual64::new(dx0, dx1));
        Ok(b)
    }
}

impl<S: Data<Elem = HyperDual64>> SolveDual<HyperDual64> for ArrayBase<S, Ix2> {
    /// Solves a system of linear equations `A * x = b` where `A` is `self`, `b`
    /// is the argument, and `x` is the successful result.
    /// ```
    /// # use approx::assert_abs_diff_eq;
    /// # use num_hyperdual::HyperDual64;
    /// # use num_hyperdual::linalg::SolveDual;
    /// # use ndarray::{arr1, arr2};
    /// let a = arr2(&[[HyperDual64::new(1.0, 2.0, 3.0, 4.0), HyperDual64::new(2.0, 3.0, 4.0, 5.0)],
    ///                [HyperDual64::new(3.0, 4.0, 5.0, 6.0), HyperDual64::new(4.0, 5.0, 6.0, 7.0)]]);
    /// let b = arr1(&[HyperDual64::new(5.0, 16.0, 22.0, 64.0), HyperDual64::new(11.0, 32.0, 42.0, 112.0)]);
    /// let x = a.solve_into(b).unwrap();
    /// assert_abs_diff_eq!((x[0].re, 1.0, epsilon = 1e-14);
    /// assert_abs_diff_eq!(x[0].eps1, 2.0, epsilon = 1e-14);
    /// assert_abs_diff_eq!(x[0].eps2, 3.0, epsilon = 1e-14);
    /// assert_abs_diff_eq!(x[0].eps1eps2, 4.0, epsilon = 1e-14);
    /// assert_abs_diff_eq!(x[1].re, 2.0, epsilon = 1e-14);
    /// assert_abs_diff_eq!(x[1].eps1, 3.0, epsilon = 1e-14);
    /// assert_abs_diff_eq!(x[1].eps2, 4.0, epsilon = 1e-14);
    /// assert_abs_diff_eq!(x[1].eps1eps2, 5.0, epsilon = 1e-14);
    /// ```
    fn solve_inplace<'a, Sb: DataMut<Elem = HyperDual64>>(
        &self,
        b: &'a mut ArrayBase<Sb, Ix1>,
    ) -> Result<&'a mut ArrayBase<Sb, Ix1>> {
        let s1 = self.mapv(|s| s.eps1);
        let s2 = self.mapv(|s| s.eps2);
        let s12 = self.mapv(|s| s.eps1eps2);
        let f = self.map(HyperDual64::re).factorize_into()?;
        let dx0 = f.solve_into(b.map(HyperDual64::re))?;
        let dx1 = f.solve_into(b.mapv(|b| b.eps1) - s1.dot(&dx0))?;
        let dx2 = f.solve_into(b.mapv(|b| b.eps2) - s2.dot(&dx0))?;
        let dx12 =
            f.solve_into(b.mapv(|b| b.eps1eps2) - s1.dot(&dx2) - s2.dot(&dx1) - s12.dot(&dx0))?;
        Zip::from(&dx0)
            .and(&dx1)
            .and(&dx2)
            .and(&dx12)
            .and(&mut *b)
            .apply(|&dx0, &dx1, &dx2, &dx12, b| *b = HyperDual64::new(dx0, dx1, dx2, dx12));
        Ok(b)
    }
}

impl<S: Data<Elem = HD3_64>> SolveDual<HD3_64> for ArrayBase<S, Ix2> {
    /// Solves a system of linear equations `A * x = b` where `A` is `self`, `b`
    /// is the argument, and `x` is the successful result.
    /// ```
    /// # use approx::assert_abs_diff_eq;
    /// # use num_hyperdual::HD3_64;
    /// # use num_hyperdual::linalg::SolveDual;
    /// # use ndarray::{arr1, arr2};
    /// let a = arr2(&[[HD3_64::new([1.0, 2.0, 3.0, 4.0]), HD3_64::new([2.0, 3.0, 4.0, 5.0])],
    ///                [HD3_64::new([3.0, 4.0, 5.0, 6.0]), HD3_64::new([4.0, 5.0, 6.0, 7.0])]]);
    /// let b = arr1(&[HD3_64::new([5.0, 16.0, 48.0, 136.0]), HD3_64::new([11.0, 32.0, 88.0, 232.0])]);
    /// let x = a.solve_into(b).unwrap();
    /// assert_abs_diff_eq!(x[0].0[0], 1.0, epsilon = 1e-14);
    /// assert_abs_diff_eq!(x[0].0[1], 2.0, epsilon = 1e-14);
    /// assert_abs_diff_eq!(x[0].0[2], 3.0, epsilon = 1e-14);
    /// assert_abs_diff_eq!(x[0].0[3], 4.0, epsilon = 1e-14);
    /// assert_abs_diff_eq!(x[1].0[0], 2.0, epsilon = 1e-14);
    /// assert_abs_diff_eq!(x[1].0[1], 3.0, epsilon = 1e-14);
    /// assert_abs_diff_eq!(x[1].0[2], 4.0, epsilon = 1e-14);
    /// assert_abs_diff_eq!(x[1].0[3], 5.0, epsilon = 1e-14);
    /// ```
    fn solve_inplace<'a, Sb: DataMut<Elem = HD3_64>>(
        &self,
        b: &'a mut ArrayBase<Sb, Ix1>,
    ) -> Result<&'a mut ArrayBase<Sb, Ix1>> {
        let s1 = self.mapv(|s| s.0[1]);
        let s2 = self.mapv(|s| s.0[2]);
        let s3 = self.mapv(|s| s.0[3]);
        let f = self.map(HD3_64::re).factorize_into()?;
        let dx0 = f.solve_into(b.map(HD3_64::re))?;
        let dx1 = f.solve_into(b.mapv(|b| b.0[1]) - s1.dot(&dx0))?;
        let dx2 = f.solve_into(b.mapv(|b| b.0[2]) - s2.dot(&dx0) - 2.0 * s1.dot(&dx1))?;
        let dx3 = f.solve_into(
            b.mapv(|b| b.0[3]) - s3.dot(&dx0) - 3.0 * s2.dot(&dx1) - 3.0 * s1.dot(&dx2),
        )?;
        Zip::from(&dx0)
            .and(&dx1)
            .and(&dx2)
            .and(&dx3)
            .and(&mut *b)
            .apply(|&dx0, &dx1, &dx2, &dx3, b| *b = HD3_64::new([dx0, dx1, dx2, dx3]));
        Ok(b)
    }
}