# rust-struct-to-wgsl

If you've looked at the other examples (which you should if you haven't already as this example skips over a lot of things covered by other examples), so far, we've only ever sent either arrays or very simple structures. Additionally, [the main example that deals with the uniform memory space](../uniform-values/README.md) says very little about special additional rules that govern how items should be layed out in buffers used via the uniform memory space.

This example not only provides examples of formatting byte buffers to be sent to the GPU but can act as a guide on how to do so. This example contains 4 sort of sub-examples, Rust structures to be translated to and from WGSL, each of which will be sent to the GPU in a buffer to be split into their component parts and placed into several member buffers that are then used in Rust to reconstruct a clone of the original struct. These can be seen as either tutorials-by-example or 'practice problems' after reading up the provided pamphlet, "[serialization_to_wgsl_for_dummies.md](./serialization_to_wgsl_for_dummies.md)".

## How to Structure a Buffer for WGSL

This README offloads this enormous subject to a companion file, [serialization_to_wgsl_for_dummies.md](./serializtion_to_wgsl_for_dummies.md). The file acts as 1. A guide to how WGSL structures its structures in memory and thus how you have to format your data in buffers for it to get translated correctly and 2. A sort of tutorial on how to translate a desired WGSL struct to a Rust representation and how to correctly place each member of the Rust equivalent into a WGPU `Buffer` to be correctly read by WGSL. It is highly recommended you at least skim it, especially if you want to follow along with the sub-example structures described briefly above and below in detail and use them as exercises for Serialization to WGSL for Dummies.

## Structure of this Example

The meat and bones can all be found in [structs.rs](./src/structs.rs). At the top of the file are the structures themselves. You can read them and try guessing for yourself how the structure will be layed out in a buffer.

Below their definitions is the definition and then implementations of `AsWgslBytes`, a trait whose only member function is meant to transform an instance of the struct into a `Vec` of bytes that can be copied into a WGPU buffer and read as the corresponding structure in WGSL. The implementations for each structure are well-commented on the serialization of each member and the justification of each time we add padding.

Below that is another trait, FromWgslBuffers and its implementations for each member. This trait isn't as important and simply uses an array of WGPU `Buffer`s which are assumed to contain each of the members in order to construct a clone of the original struct. Feel free to check it out if you want though, especially if you're interested in, say, what turning a `&[u8]` into a `[[f32; 2]; 4]` with a single expression in Rust looks like.

The shaders themselves aren't that interesting. The main reason to look at them is to see what the structs look like in WGSL. Other than that, they're basically all the same in principal.

[utils.rs](./src/utils.rs) contains helper functions including the major code implemented generically for the running of the example for each struct (`ExampleStruct::run_as_example`).