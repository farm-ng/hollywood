/// Trait for actor state and props.
pub trait Value: Default + std::fmt::Debug + Send + Sync + Clone + 'static {}


/// Empty state - for stateless actors.
#[derive(Clone, Debug, Default)]
pub struct NullState {}

impl Value for NullState {}

/// Empty prop - for actors without props.
#[derive(Clone, Debug, Default)]
pub struct NullProp {}

impl Value for NullProp {}
