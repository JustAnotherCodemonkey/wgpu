// Try looking at the structs and guessing, yourself, what we will do to format it
// into bytes for WGSL. The solutions with explanations are just below the structs
// themselves.

use pollster::FutureExt;

use super::utils::{get_bytes_from_buffer, validate_buffers_for_from_wgsl_bytes};

/// This should be easy to do.
#[derive(Debug, Clone, PartialEq)]
pub struct Beginner {
    pub a: i32,
    /// Translates to vec2<f32>
    pub b: [f32; 2],
}

/// This one a little bit more tricky.
#[derive(Debug, Clone, PartialEq)]
pub struct Intermediate {
    pub a: i32,
    /// Translates to vec3<f32>
    pub b: [f32; 3],
    /// Translates to vec2<i32>
    pub c: [i32; 2],
}

/// ✨ Advanced. ✨ If this one is intuitive to you, you should be able
/// to handle anything.
#[derive(Debug, Clone, PartialEq)]
pub struct Advanced {
    pub a: u32,
    /// Translates to array<i32, 3>
    pub b: [i32; 3],
    /// Translates to AdvancedInner
    pub c: AdvancedInner,
    pub d: i32,
}

/// Goes inside [`Advanced`] for extra fun.
#[derive(Debug, Clone, PartialEq)]
pub struct AdvancedInner {
    /// Translates to vec2<i32>
    pub a: [i32; 2],
    /// Translates to mat4x2<f32>
    pub b: [[f32; 2]; 4],
    pub c: i32,
}

/// Look alive for this curve ball, soldier! This one will be placed the Uniform
/// memory space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InUniform {
    /// Translates to InUniformInner.
    pub a: InUniformInner,
    /// Has align(16)
    pub b: i32,
    /// Translates to array<i32_wrapper, 2> where i32_wrapper is defined as
    /// ```wgsl
    /// struct i32_wrapper {
    ///     @size(16)
    ///     elem: i32,
    /// }
    /// Has align(16)
    pub c: [i32; 2],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InUniformInner {
    pub a: i32,
    pub b: i32,
}

pub trait AsWgslBytes {
    fn as_wgsl_bytes(&self) -> Vec<u8>;
}

// Solutions -------------------------------------------------

impl AsWgslBytes for Beginner {
    fn as_wgsl_bytes(&self) -> Vec<u8> {
        // We'll use this to store our bytes and return this.
        let mut bytes = Vec::<u8>::new();

        // a
        // `a` is pretty easy. As an i32, it has a size of 4 and an align of
        // 4 as well. Since this is the start of the structure, we don't
        // need to do any aligning.
        //
        // Just remember that in WGSL, numbers are stored in little endian
        // format.
        bytes.extend_from_slice(&self.a.to_le_bytes());

        // b
        // `b` is where things get a little tricky. `b` has a size of 8 and
        // an align of 8. Remember that we are at offset 4 so placing
        // `b` right now would be out of alignment. We need to introduce padding
        // first in order to put `b` at a multiple of 8. Since we're at 4, we
        // need 4 more bytes. Let's add those now.
        bytes.resize(bytes.len() + 4, 0);
        // Now we can add the bytes of b.
        bytes.extend(self.b.iter().flat_map(|v| v.to_le_bytes()));

        // And our struct is now sized at a multiple of it's alignment
        // (8 because its largest member, `b`, has an alignment of 8).
        // We can now return our formatted struct bytes.
        bytes
    }
}

impl AsWgslBytes for Intermediate {
    fn as_wgsl_bytes(&self) -> Vec<u8> {
        // Just like with Beginner in a lot of ways.
        let mut bytes = Vec::<u8>::new();

        // a
        // Once again, no alignment is needed.
        bytes.extend_from_slice(&self.a.to_le_bytes());

        // b
        // `b` has an alignment of 16 (size of 12, rounded to nearest
        // power of 2) but we're at offset 4 so we need to pad
        // to the next multiple of 16 to align for `b`. That multiple
        // happens to be 16 so let's pad to there.
        bytes.resize(16, 0);
        // Now, push the actual bytes of `b`
        for v in self.b.iter() {
            bytes.extend_from_slice(&v.to_le_bytes());
        }

        // c
        // Now, `c` has an alignment of 8 but because `b` was only 12 bytes long,
        // we're currently at byte 28, which is not a multiple of 8. 4 more
        // bytes and we'll be there but here's a neat trick to get the next
        // multiple of a number:
        let new_desired_len = (bytes.len() / 8 + 1) * 8;
        // Note that this can get you into trouble as it will always bump
        // you ahead at least by a multiple so optimally you would want to
        // check first if a bump is necessary.
        bytes.resize(new_desired_len, 0);
        // Now we can add `c`. Let's do it differently for fun.
        bytes.extend_from_slice(&self.c[0].to_le_bytes());
        bytes.extend_from_slice(&self.c[1].to_le_bytes());

        // Now, we've got all our members but if you've read the README well,
        // you should know that there's one last thing we need to do. The struct's
        // size must be a multiple of its alignment. Since the member of with the
        // largest alignment has an alignment of 16, the struct does too.
        //
        // Let's use our trick again to align to 16 and extend our struct with
        // end-of-struct struct size padding.
        if bytes.len() % 16 != 0 {
            bytes.resize((bytes.len() / 16 + 1) * 16, 0);
        }

        bytes
    }
}

impl AsWgslBytes for Advanced {
    fn as_wgsl_bytes(&self) -> Vec<u8> {
        // Hopefully by now, you're getting the hang of this.
        let mut bytes = Vec::<u8>::new();

        // a
        bytes.extend_from_slice(&self.a.to_le_bytes());

        // b
        // You may instinctively think at this point that we need to align `b`
        // but here, it's actually already aligned. The arrays we were working
        // with before were translated into vec's but `b` will be an array.
        // While vec's are aligned as one unit, arrays have the align of their
        // element type. `b`'s alignment is 4 just like a so it can start right
        // after it.
        for v in self.b.iter() {
            bytes.extend_from_slice(&v.to_le_bytes());
        }

        // c
        // `c` is where things get interesting. First, let's make sure we're aligned
        // (we are but our trick works anyways).
        if bytes.len() % 8 != 0 {
            bytes.resize((bytes.len() / 8 + 1) * 8, 0);
        }
        // Because AdvancedInner's `a` and `b`
        // are going to come next in memory logically, let's jump over to
        // AdvancedInner's impl of AsWgslBytes.
        bytes.extend_from_slice(&self.c.as_wgsl_bytes());

        // d
        // Finally, `d`. We're aligned but assuming alignment is so for beginners.
        // This is Rust and we are big kids and big kids write safe code.
        // Well, big kids also write functions for code they reuse but...
        if bytes.len() % 4 != 0 {
            bytes.resize((bytes.len() / 4 + 1) * 4, 0);
        }
        bytes.extend_from_slice(&self.d.to_le_bytes());

        // Almost done. We just need to pad the struct. As big as is,
        // it still only has an align of 8. We will need it but our trick
        // means that we don't care either way.
        if bytes.len() % 8 != 0 {
            bytes.resize((bytes.len() / 8 + 1) * 8, 0);
        }

        // Aaaand we're done! Whew, pat yourself on the back if you got all that!
        bytes
    }
}

impl AsWgslBytes for AdvancedInner {
    fn as_wgsl_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();

        // a
        for v in self.a.iter() {
            bytes.extend_from_slice(&v.to_le_bytes());
        }

        // b
        // Now `b` is where things get super interesting. You once again might
        // be tempted to start calculating alignment but here, there is actually
        // no need. If you took a look at the chart on the wgsl spec, you would
        // see that matrices have the alignment of their rows. The spec even
        // alludes to the idea that matrices are stored as array<vecR<T>, C>.
        //
        // Considering this, we'll check the alignment. `a` had an alignment of 8
        // and since `b`'s row alignment is also 8, no padding is necessary.
        //
        // Nothing fancy here, just dump the bytes.
        for row in self.b.iter() {
            for v in row.iter() {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
        }

        // c
        // `c`, as we'll see, throws a very small wrench into things.
        bytes.extend_from_slice(&self.c.to_le_bytes());

        // Now, before `c`, we had an item with a size of 32 slot perfectly with
        // an item with a size of 8, making a total size of 40. And with an
        // alignment of 8, this struct could have been done. However, with `c` in
        // the mix, adding 4 means the structure's size is no longer a multiple
        // of its alignment, 8. We need to pad 4 bytes to bring its size up to 48.
        //
        // The main point of this inner structure (besides demonstrating the thing
        // with matrices,) is to show struct size padding in action.
        bytes.resize(bytes.len() + 4, 0);

        bytes
    }
}

impl AsWgslBytes for InUniform {
    fn as_wgsl_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();

        // a
        // It's the start of the struct, what do you expect?
        bytes.extend_from_slice(&self.a.as_wgsl_bytes());

        // b
        // This is where things get tricky. Although i32 has an align of 4,
        // if you saw the shader code, you'd see that `b` is marked with an
        // align attribute setting its align to 16. This is because of one
        // of the rules about structure members in the uniform memory space:
        // if one of the members is a structure, all subsequent members
        // of that structure must have an alignment that is a multiple of 16.
        // We had `a` and now we need to set the alignment of everything to
        // 16.
        //
        // Lets use our trick from `Advanced`, which we've turned into
        // a function.
        align_vec(&mut bytes, 16);
        bytes.extend_from_slice(&self.b.to_le_bytes());

        // c
        // c is tricky as an array. First, let's get the alignment out of
        // the way. Just like `b`, `c` needs to have an alignment that is
        // a multiple of 16 because it comes after a structure in the
        // parent structure's member list but it also needs to have an
        // alignment that is a multiple of 16 by virtue of being an array
        // in the uniform memory space.
        //
        // There is also the issue of the array elements. Remember, in
        // the uniform memory space, array elements must also be aligned
        // at 16 byte boundaries.
        align_vec(&mut bytes, 16);
        for e in self.c.iter() {
            bytes.extend_from_slice(&e.to_le_bytes());
            // Here is where we pad the element out to the next 16 byte
            // boundary, simulating the `@size(16)` we see in the shader
            // code.
            align_vec(&mut bytes, 16);
        }

        // Finally, struct padding. Largest element align was 16 so we
        // pad to multiple of 16. We know because of the above code
        // that it is already aligned but we are in the larger leagues
        // now so we should practice safe code.
        align_vec(&mut bytes, 16);

        bytes
    }
}

impl AsWgslBytes for InUniformInner {
    fn as_wgsl_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();

        // a
        bytes.extend_from_slice(&self.a.to_le_bytes());

        // b
        // Note that we don't need to align to 16 here because none of the conditions
        // where alignment to 16 is necessary apply.
        bytes.extend_from_slice(&self.b.to_le_bytes());

        bytes
    }
}

/// Utility for converting from a series of member buffers into an example struct.
pub trait FromWgslBuffers {
    fn desired_buffer_sizes() -> Vec<u64>;

    fn from_wgsl_buffers(
        buffers: &[&wgpu::Buffer],
        staging_buffer: &wgpu::Buffer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self;
}

impl FromWgslBuffers for Beginner {
    fn desired_buffer_sizes() -> Vec<u64> {
        vec![4, 8]
    }

    fn from_wgsl_buffers(
        buffers: &[&wgpu::Buffer],
        staging_buffer: &wgpu::Buffer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        #[cfg(debug_assertions)]
        validate_buffers_for_from_wgsl_bytes::<Self>(
            buffers,
            &Beginner::desired_buffer_sizes(),
            staging_buffer,
        );

        let a = i32::from_le_bytes(
            <[u8; 4]>::try_from(
                get_bytes_from_buffer(buffers[0], staging_buffer, device, queue)
                    .block_on()
                    .as_slice(),
            )
            .unwrap(),
        );
        let b = <[f32; 2]>::try_from(
            get_bytes_from_buffer(buffers[1], staging_buffer, device, queue)
                .block_on()
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes(<[u8; 4]>::try_from(chunk).unwrap()))
                .collect::<Vec<f32>>()
                .as_slice(),
        )
        .unwrap();

        Beginner { a, b }
    }
}

impl FromWgslBuffers for Intermediate {
    fn desired_buffer_sizes() -> Vec<u64> {
        vec![4, 12, 8]
    }

    fn from_wgsl_buffers(
        buffers: &[&wgpu::Buffer],
        staging_buffer: &wgpu::Buffer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        #[cfg(debug_assertions)]
        validate_buffers_for_from_wgsl_bytes::<Self>(
            buffers,
            &Intermediate::desired_buffer_sizes(),
            staging_buffer,
        );

        let a = i32::from_le_bytes(
            <[u8; 4]>::try_from(
                get_bytes_from_buffer(buffers[0], staging_buffer, device, queue)
                    .block_on()
                    .as_slice(),
            )
            .unwrap(),
        );
        let b = <[f32; 3]>::try_from(
            get_bytes_from_buffer(buffers[1], staging_buffer, device, queue)
                .block_on()
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes(<[u8; 4]>::try_from(chunk).unwrap()))
                .collect::<Vec<f32>>()
                .as_slice(),
        )
        .unwrap();
        let c = <[i32; 2]>::try_from(
            get_bytes_from_buffer(buffers[2], staging_buffer, device, queue)
                .block_on()
                .chunks_exact(4)
                .map(|chunk| i32::from_le_bytes(<[u8; 4]>::try_from(chunk).unwrap()))
                .collect::<Vec<i32>>()
                .as_slice(),
        )
        .unwrap();

        Intermediate { a, b, c }
    }
}

impl FromWgslBuffers for AdvancedInner {
    fn desired_buffer_sizes() -> Vec<u64> {
        vec![8, 32, 4]
    }

    fn from_wgsl_buffers(
        buffers: &[&wgpu::Buffer],
        staging_buffer: &wgpu::Buffer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        #[cfg(debug_assertions)]
        validate_buffers_for_from_wgsl_bytes::<AdvancedInner>(
            buffers,
            &AdvancedInner::desired_buffer_sizes(),
            staging_buffer,
        );

        let a = <[i32; 2]>::try_from(
            get_bytes_from_buffer(buffers[0], staging_buffer, device, queue)
                .block_on()
                .chunks_exact(4)
                .map(|chunk| i32::from_le_bytes(<[u8; 4]>::try_from(chunk).unwrap()))
                .collect::<Vec<i32>>()
                .as_slice(),
        )
        .unwrap();
        let b = <[[f32; 2]; 4]>::try_from(
            get_bytes_from_buffer(buffers[1], staging_buffer, device, queue)
                .block_on()
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes(<[u8; 4]>::try_from(chunk).unwrap()))
                .collect::<Vec<f32>>()
                .chunks_exact(2)
                .map(|chunk| <[f32; 2]>::try_from(chunk).unwrap())
                .collect::<Vec<[f32; 2]>>()
                .as_slice(),
        )
        .unwrap();
        let c = i32::from_le_bytes(
            <[u8; 4]>::try_from(
                get_bytes_from_buffer(buffers[2], staging_buffer, device, queue)
                    .block_on()
                    .as_slice(),
            )
            .unwrap(),
        );

        AdvancedInner { a, b, c }
    }
}

impl FromWgslBuffers for Advanced {
    fn desired_buffer_sizes() -> Vec<u64> {
        vec![4, 12, 8, 32, 4, 4]
    }

    /// Buffers are in order [a, b, c.a, c.b, c.c, d].
    fn from_wgsl_buffers(
        member_buffers: &[&wgpu::Buffer],
        staging_buffer: &wgpu::Buffer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        #[cfg(debug_assertions)]
        validate_buffers_for_from_wgsl_bytes::<Advanced>(
            member_buffers,
            &Advanced::desired_buffer_sizes(),
            staging_buffer,
        );

        let a = u32::from_le_bytes(
            <[u8; 4]>::try_from(
                get_bytes_from_buffer(member_buffers[0], staging_buffer, device, queue).block_on(),
            )
            .unwrap(),
        );
        let b = <[i32; 3]>::try_from(
            get_bytes_from_buffer(member_buffers[1], staging_buffer, device, queue)
                .block_on()
                .chunks_exact(4)
                .map(|chunk| i32::from_le_bytes(<[u8; 4]>::try_from(chunk).unwrap()))
                .collect::<Vec<i32>>()
                .as_slice(),
        )
        .unwrap();
        let c =
            AdvancedInner::from_wgsl_buffers(&member_buffers[2..5], staging_buffer, device, queue);
        let d = i32::from_le_bytes(
            <[u8; 4]>::try_from(
                get_bytes_from_buffer(member_buffers[5], staging_buffer, device, queue).block_on(),
            )
            .unwrap(),
        );

        Advanced { a, b, c, d }
    }
}

impl FromWgslBuffers for InUniform {
    fn desired_buffer_sizes() -> Vec<u64> {
        vec![4, 4, 4, 8]
    }

    fn from_wgsl_buffers(
        buffers: &[&wgpu::Buffer],
        staging_buffer: &wgpu::Buffer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        #[cfg(debug_assertions)]
        validate_buffers_for_from_wgsl_bytes::<InUniform>(
            buffers,
            &InUniform::desired_buffer_sizes(),
            staging_buffer,
        );

        InUniform {
            a: InUniformInner {
                a: i32::from_le_bytes(
                    <[u8; 4]>::try_from(
                        get_bytes_from_buffer(buffers[0], staging_buffer, device, queue).block_on(),
                    )
                    .unwrap(),
                ),
                b: i32::from_le_bytes(
                    <[u8; 4]>::try_from(
                        get_bytes_from_buffer(buffers[1], staging_buffer, device, queue).block_on(),
                    )
                    .unwrap(),
                ),
            },
            b: i32::from_le_bytes(
                <[u8; 4]>::try_from(
                    get_bytes_from_buffer(buffers[2], staging_buffer, device, queue).block_on(),
                )
                .unwrap(),
            ),
            c: <[i32; 2]>::try_from(
                get_bytes_from_buffer(buffers[3], staging_buffer, device, queue)
                    .block_on()
                    .chunks_exact(4)
                    .map(|chunk| i32::from_le_bytes(<[u8; 4]>::try_from(chunk).unwrap()))
                    .collect::<Vec<i32>>()
                    .as_slice(),
            )
            .unwrap(),
        }
    }
}

fn align_vec(v: &mut Vec<u8>, alignment: usize) {
    if v.len() % alignment != 0 {
        v.resize((v.len() / alignment + 1) * alignment, 0);
    }
}
