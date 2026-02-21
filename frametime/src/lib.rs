//! Collect frametimes and produce common rendering
//! statistics based on them.
use std::fmt;

/// Store frametimes in a statically allocated
/// ring buffer.
///
/// The ring buffer is initialized with all zeroes, making
/// the internal state valid from the start.
///
/// The frametimes are stored in nanoseconds, providing
/// the highest possible precision.
///
/// `SAMPLES` defines the number of samples the buffer will hold.
/// This buffer size does not need to match in any way how many
/// frames are actually generated within a second.
/// Depending on the actual framerate the `Sampler` may not be able
/// to contain all the frametimes for a given second. Or it may
/// be able to contain the frametimes for multiple seconds.
/// It won't really matter. Given that the frametimes themselves
/// are collected the underlying FPS count may always be approximated.
///
/// Powers of two from '64' should provide decent metrics.
/// The more the better of course, but more than '1024' should no be
/// necessary as the returns on extra precision will be rapidly diminishing.
#[derive(Debug, Clone, Copy)]
pub struct Sampler<const SAMPLES: usize> {
    frametimes: [u128; SAMPLES],
    entry_idx: usize,
}

impl<const SAMPLES: usize> Default for Sampler<SAMPLES> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SAMPLES: usize> Sampler<SAMPLES> {
    const ZERO_POINT_ONE_PERCENT_SAMPLE_COUNT: usize = ((SAMPLES as f64) * 0.001) as usize;
    const ONE_PERCENT_SAMPLE_COUNT: usize = ((SAMPLES as f64) * 0.01) as usize;

    pub fn new() -> Sampler<SAMPLES> {
        Sampler {
            frametimes: [0; SAMPLES],
            entry_idx: 0,
        }
    }

    /// Record a frametime into the buffer.
    ///
    /// The frametimes are expected to be provided in
    /// nanoseconds.
    pub fn add_frametime(&mut self, frametime: u128) {
        self.entry_idx = (self.entry_idx + 1) % SAMPLES;
        self.frametimes[self.entry_idx] = frametime;
    }

    /// Generate the FPS `Stats`.
    ///
    /// For any accurate results the `Sampler`
    /// will have to be continuously fed with data.
    /// Until the ring buffer is fully filled first,
    /// otherwise the initial zeroes will skew the
    /// statistics.
    ///
    /// This operation can affect the collected data
    /// itself. If that may be a problem, use the
    /// `Sampler` in a different thread.
    pub fn stats(&self) -> Stats {
        let data = {
            let mut data = self.frametimes;
            data.sort_unstable_by(|lhs, rhs| rhs.cmp(lhs));
            data
        };

        let mut zero_point_one_percent_average = 0.0;
        let mut ninety_nine_point_ninth_percentile = 0.0;
        let mut one_percent_average = 0.0;
        let mut ninety_ninth_percentile = 0.0;
        let mut average = 0.0;
        let mut sum = 0;

        for (i, value) in data.iter().enumerate() {
            sum += value;

            // Frametimes are converted into FPS counts.
            //
            // The formula is:
            // average fps = 1.0 / ((frametime sum / 10^9) / sample count)
            // FPS is frames / second and we have frame times in nanoseconds so
            // first we convert the number to seconds in `(frametime sum / 10^9)`.
            // Then dividing this by `sample count` we get an `average frametimes in seconds` value.
            // This is then turned into FPS by taking its inverse: 1 / `average frametimes in seconds`.
            //
            // To reduce the number of brackets the equation was simplified:
            // 1.0 / ((frametime sum / 10^9) / sample count) =>
            // sample count / (frametime sum / 10^9) =>
            // sample count * 10^9 / frametime sum
            //
            // Perhaps we could consider ordering the operations in such a way that the intermediate floating point
            // values stay in the lower value range, increasing floating point precision, but this would not
            // worth the effort of describing the why and hows, nor would it provide too much benefit in these
            // calculations.
            if i == Sampler::<SAMPLES>::ZERO_POINT_ONE_PERCENT_SAMPLE_COUNT - 1 {
                zero_point_one_percent_average =
                    Sampler::<SAMPLES>::ZERO_POINT_ONE_PERCENT_SAMPLE_COUNT as f64 * 10_f64.powi(9)
                        / sum as f64;
                ninety_nine_point_ninth_percentile = *value as f64;
            } else if i == Sampler::<SAMPLES>::ONE_PERCENT_SAMPLE_COUNT - 1 {
                one_percent_average = Sampler::<SAMPLES>::ONE_PERCENT_SAMPLE_COUNT as f64
                    * 10_f64.powi(9)
                    / sum as f64;
                ninety_ninth_percentile = *value as f64;
            } else if i == SAMPLES - 1 {
                average = SAMPLES as f64 * 10_f64.powi(9) / sum as f64;
            }
        }

        Stats {
            average,
            one_percent_average,
            zero_point_one_percent_average,
            ninety_ninth_percentile,
            ninety_nine_point_ninth_percentile,
        }
    }
}

/// FPS/Frametime `Stats`
///
/// It does not provide the minimum or maximum
/// FPS counts, because for all practical purposes
/// those are entirely useless.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Stats {
    average: f64,
    one_percent_average: f64,
    zero_point_one_percent_average: f64,
    ninety_ninth_percentile: f64,
    ninety_nine_point_ninth_percentile: f64,
}

impl Stats {
    /// Get the average FPS count.
    pub fn average(&self) -> f64 {
        self.average
    }

    /// Get the average FPS count from the slowest
    /// 1% of frametimes.
    pub fn one_percent_lows_average(&self) -> f64 {
        self.one_percent_average
    }

    /// Get the average FPS count from the slowest 0.1% of
    /// frametimes.
    pub fn zero_point_one_percent_lows_average(&self) -> f64 {
        self.zero_point_one_percent_average
    }

    /// Get the 99 percentile lowest frame time.
    ///
    /// The returned value means that 99% of the frames
    /// in the sampling interval were faster than this
    /// frame.
    pub fn ninety_ninth_percentile(&self) -> f64 {
        self.ninety_ninth_percentile
    }

    /// Get the 99.9 percentile lowest frame time.
    ///
    /// The returned value means that 99.9% of the frames
    /// in the sampling interval were faster than this
    /// frame.
    pub fn ninety_nine_point_nine_percentile(&self) -> f64 {
        self.ninety_nine_point_ninth_percentile
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Avg. FPS: {:.2}\n1% low: {:.2}\n0.1% low: {:.2}\n99 th: {} ns\n99.9 th: {} ns\n",
            self.average(),
            self.one_percent_lows_average(),
            self.zero_point_one_percent_lows_average(),
            self.ninety_ninth_percentile(),
            self.ninety_nine_point_nine_percentile(),
        )
    }
}
