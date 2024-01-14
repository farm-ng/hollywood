//! Examples of hollywood actor framework.

/// Moving average example
pub mod moving_average;

/// One dimensional robot Kalman filter example.
/// 
/// ```text
/// *   Periodic_0   *
/// |   time_stamp   |
///         ⠉⠉⠉⠉⠑⠒⠒⠒⠒⠤⠤⠤⠤⢄⣀⣀⣀⣀
///                           ⠉⠉⠉⠉⠑⠒⠒⠒⠒⠤⠤⠤⠤⢄⣀⣀⣀⣀
///                                             ⠁
///                                     |   TimeStamp    |
///                                     *     Sim_0      *
/// | cancel_request     noisy_range      noisy_velocity      true_range        true_robot      true_velocity  |
///                  ⢀⣀⡠⠤⠤⠒⠒⠉⠉⠁        ⢀⣀⡠⠤⠤⠒⠒⠉⠉⠁                 ⣇⣀⣀⣀⣀⠤⠤⠤⠤⠔⠒⠒⠒⠒⣉⠭⠛⠉⠁
///         ⢀⣀⡠⠤⠤⠒⠒⠉⠉⠁        ⢀⣀⡠⠤⠤⠒⠒⠉⠉⠁        ⢀⣀⣀⣀⣀⠤⠤⠤⠤⠔⠒⠒⠒⠒⠉⠉⠉⠉⡇        ⢀⡠⠔⠊⠉
///         ⠁                 ⠁                 ⠁                 ⡇    ⣀⠤⠒⠉⠁
/// |   NoisyRange      NoisyVelocity  ||   Printable    |        ⣇⡠⠔⠊⠉
/// *    Filter_0    *                  *Printer(truth)_0*    ⣀⠤⠒⠉⡇
/// |predicted_state    updated_state  |                 ⢀⡠⠔⠊⠉    ⡇
///                  ⢀⣀⡠⠤⠤⠒⠒⠉⠉⡇                      ⣀⠤⠒⠉⠁        ⡇
///         ⢀⣀⡠⠤⠤⠒⠒⠉⠉⠁        ⡇                 ⢀⡠⠔⠊⠉             ⡇
///         ⠁                 ⠁                 ⠁                 ⠁
/// |   Printable    ||   FilterEst          TruePos          TrueRange    |
/// *Printer(filter s*                  *  DrawActor_0   *
/// ```
pub mod one_dim_robot;


