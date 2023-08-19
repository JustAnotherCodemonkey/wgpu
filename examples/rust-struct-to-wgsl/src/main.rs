mod structs;
#[cfg(test)]
mod tests;
mod utils;

use pollster::FutureExt;
use structs::{
    Advanced, AdvancedInner, AsWgslBytes, Beginner, FromWgslBuffers, InUniform, InUniformInner,
    Intermediate,
};
use utils::{
    compute, create_bind_group, create_input_buffer, create_output_buffers, create_pipeline,
    create_staging_buffer, SystemContext,
};

static BEGINNER_SHADER: &str = include_str!("beginner.wgsl");
static INTERMEDIATE_SHADER: &str = include_str!("intermediate.wgsl");
static ADVANCED_SHADER: &str = include_str!("advanced.wgsl");
static IN_UNIFORM_SHADER: &str = include_str!("in-uniform.wgsl");

fn main() {
    todo!();
}
