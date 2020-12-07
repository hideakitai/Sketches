use super::binding::Binding;

pub struct Material {
    pub name: String,
    // TODO: make texture binding to Arc and refer it??
    pub binding: Binding,
}
