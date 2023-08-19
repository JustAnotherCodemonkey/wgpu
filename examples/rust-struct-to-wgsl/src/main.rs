mod structs;
#[cfg(test)]
mod tests;
mod utils;

use structs::{
    Advanced, AdvancedInner, AsWgslBytes, Beginner, FromWgslBuffers, InUniform, InUniformInner,
    Intermediate,
};
use utils::{
    compute, create_bind_group, create_input_buffer, create_output_buffers, create_pipeline,
    create_staging_buffer, SystemContext,
};
use pollster::FutureExt;

fn main() {
    todo!();
}
