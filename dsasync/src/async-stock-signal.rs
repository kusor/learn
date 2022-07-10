use async_trait::async_trait;
///
/// A trait to provide a common interface for all signal calculations.
///
#[async_trait]
pub trait AsyncStockSignal {
    ///
    /// The signal's data type.
    ///
    type SignalType;

    ///
    /// Calculate the signal on the provided series.
    ///
    /// # Returns
    ///
    /// The signal (using the provided type) or `None` on error/invalid data.
    ///
    async fn calculate(&self, series: &[f64]) -> Option<Self::SignalType>;
}

///
/// Find the maximum in a series of f64
///
pub struct MaxPrice {}

#[async_trait]
impl AsyncStockSignal for MaxPrice {
    type SignalType = f64;

    async fn calculate(&self, series: &[f64]) -> Option<Self::SignalType> {
        if series.is_empty() {
            None
        } else {
            Some(series.iter().fold(f64::MIN, |acc, q| acc.max(*q)))
        }
    }
}

///
/// Calculates the absolute and relative difference between the beginning and ending of an f64 series. The relative difference is relative to the beginning.
///
pub struct PriceDifference {}

#[async_trait]
impl AsyncStockSignal for PriceDifference {
    type SignalType = (f64, f64);

    /// # Returns
    ///
    /// A tuple `(absolute, relative)` difference.
    ///
    async fn calculate(&self, a: &[f64]) -> Option<(f64, f64)> {
        if !a.is_empty() {
            // unwrap is safe here even if first == last
            let (first, last) = (a.first().unwrap(), a.last().unwrap());
            let abs_diff = last - first;
            let first = if *first == 0.0 { 1.0 } else { *first };
            let rel_diff = abs_diff / first;
            Some((abs_diff, rel_diff))
        } else {
            None
        }
    }
}

///
/// Window function to create a simple moving average
///
pub struct WindowedSMA {
    pub window_size: usize,
}

#[async_trait]
impl AsyncStockSignal for WindowedSMA {
    type SignalType = Vec<f64>;

    async fn calculate(&self, series: &[f64]) -> Option<Vec<f64>> {
        if !series.is_empty() && self.window_size > 1 {
            Some(
                series
                    .windows(self.window_size)
                    .map(|w| w.iter().sum::<f64>() / w.len() as f64)
                    .collect(),
            )
        } else {
            None
        }
    }
}

///
/// Find the minimum in a series of f64
///
pub struct MinPrice {}

#[async_trait]
impl AsyncStockSignal for MinPrice {
    type SignalType = f64;

    async fn calculate(&self, series: &[f64]) -> Option<f64> {
        if series.is_empty() {
            None
        } else {
            Some(series.iter().fold(f64::MAX, |acc, q| acc.min(*q)))
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;

    #[async_std::test]
    async fn test_PriceDifference_calculate() -> std::io::Result<()> {
        let signal = PriceDifference {};
        assert_eq!(signal.calculate(&[]).await, None);
        assert_eq!(signal.calculate(&[1.0]).await, Some((0.0, 0.0)));
        assert_eq!(signal.calculate(&[1.0, 0.0]).await, Some((-1.0, -1.0)));
        assert_eq!(
            signal
                .calculate(&[2.0, 3.0, 5.0, 6.0, 1.0, 2.0, 10.0])
                .await,
            Some((8.0, 4.0))
        );
        assert_eq!(
            signal.calculate(&[0.0, 3.0, 5.0, 6.0, 1.0, 2.0, 1.0]).await,
            Some((1.0, 1.0))
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_MinPrice_calculate() -> std::io::Result<()> {
        let signal = MinPrice {};
        assert_eq!(signal.calculate(&[]).await, None);
        assert_eq!(signal.calculate(&[1.0]).await, Some(1.0));
        assert_eq!(signal.calculate(&[1.0, 0.0]).await, Some(0.0));
        assert_eq!(
            signal
                .calculate(&[2.0, 3.0, 5.0, 6.0, 1.0, 2.0, 10.0])
                .await,
            Some(1.0)
        );
        assert_eq!(
            signal.calculate(&[0.0, 3.0, 5.0, 6.0, 1.0, 2.0, 1.0]).await,
            Some(0.0)
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_MaxPrice_calculate() -> std::io::Result<()> {
        let signal = MaxPrice {};
        assert_eq!(signal.calculate(&[]).await, None);
        assert_eq!(signal.calculate(&[1.0]).await, Some(1.0));
        assert_eq!(signal.calculate(&[1.0, 0.0]).await, Some(1.0));
        assert_eq!(
            signal
                .calculate(&[2.0, 3.0, 5.0, 6.0, 1.0, 2.0, 10.0])
                .await,
            Some(10.0)
        );
        assert_eq!(
            signal.calculate(&[0.0, 3.0, 5.0, 6.0, 1.0, 2.0, 1.0]).await,
            Some(6.0)
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_WindowedSMA_calculate() -> std::io::Result<()> {
        let series = vec![2.0, 4.5, 5.3, 6.5, 4.7];

        let signal = WindowedSMA { window_size: 3 };
        assert_eq!(
            signal.calculate(&series).await,
            Some(vec![3.9333333333333336, 5.433333333333334, 5.5])
        );

        let signal = WindowedSMA { window_size: 5 };
        assert_eq!(signal.calculate(&series).await, Some(vec![4.6]));

        let signal = WindowedSMA { window_size: 10 };
        assert_eq!(signal.calculate(&series).await, Some(vec![]));
        Ok(())
    }
}
