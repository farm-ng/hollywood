/// State of an actor.
pub trait StateTrait: Default + std::fmt::Debug + Send + Sync + Clone + 'static {}

/// Null state for stateless actors.
#[derive(Clone, Debug, Default)]
pub struct NullState {}

impl StateTrait for NullState {}
