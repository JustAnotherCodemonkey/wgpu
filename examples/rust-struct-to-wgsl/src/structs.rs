// Conversion implementations are at the very bottom of the file to avoid spoiling it.
// Try to see if you can figure out how you would do each one before you look
// at its respective function.

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
    pub a: i32,
    /// Translates to array<i32, 2>
    pub b: [i32; 2],
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
        // a is pretty easy. As an i32, it has a size of 4 and an align of
        // 4 as well. Since this is the start of the structure, we don't
        // need to do any aligning.
        //
        // Just remember that in WGSL, numbers are stored in little endian
        // format.
        bytes.extend_from_slice(&self.a.to_le_bytes());

        // b
        // b is where things get a little tricky. b has a size of 8 and
        // an align of 8. Remember that we are at offset 4 so placing
        // b right now would be out of alignment. We need to introduce padding
        // first in order to put b at a multiple of 8. Since we're at 4, we
        // need 4 more bytes. Let's add those now.
        bytes.resize(bytes.len() + 4, 0);
        // Now we can add the bytes of b.
        bytes.extend(self.b.iter().map(|v| v.to_le_bytes()).flatten());

        // And our struct is now sized at a multiple of it's alignment
        // (8 because its largest member, b, has an alignment of 8).
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
        // b has an alignment of 16 (size of 12, rounded to nearest
        // power of 2) but we're at offset 4 so we need to pad
        // to the next multiple of 16 to align for b. That multiple
        // happens to be 16 so let's pad to there.
        bytes.resize(16, 0);
        // Now, push the actual bytes of b
        for v in self.b.iter() {
            bytes.extend_from_slice(&v.to_le_bytes());
        }

        // c
        // Now, c has an alignment of 8 but because b was only 12 bytes long,
        // we're currently at byte 28, which is not a multiple of 8. 4 more
        // bytes and we'll be there but here's a neat trick to get the next
        // multiple of a number:
        let new_desired_len = (bytes.len() / 8 + 1) * 8;
        // Note that this can get you into trouble as it will always bump
        // you ahead at least by a multiple so optimally you would want to
        // check first if a bump is necessary.
        bytes.resize(new_desired_len, 0);
        // Now we can add c. Let's do it differently for fun.
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
        // You may instinctively think at this point that we need to align b
        // but here, it's actually already aligned. The arrays we were working
        // with before were translated into vec's but b will be an array.
        // While vec's are aligned as one unit, arrays have the align of their
        // element type. b's alignment is 4 just like a so it can start right
        // after it.
        for v in self.b.iter() {
            bytes.extend_from_slice(&v.to_le_bytes());
        }

        // c
        // c is where things get interesting. First, let's make sure we're aligned
        // (we are but our trick works anyways).
        if bytes.len() % 8 != 0 {
            bytes.resize((bytes.len() / 8 + 1) * 8, 0);
        }
        // Because AdvancedInner's a and b
        // are going to come next in memory logically, let's jump over to
        // AdvancedInner's impl of AsWgslBytes.
        bytes.extend_from_slice(&self.c.as_wgsl_bytes());

        // d
        // Finally, d. We're aligned but assuming alignment is so for beginners.
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
        // Now b is where things get super interesting. You once again might
        // be tempted to start calculating alignment but here, there is actually
        // no need. If you took a look at the chart on the wgsl spec, you would
        // see that matrices have the alignment of their rows. The spec even
        // alludes to the idea that matrices are stored as array<vecR<T>, C>.
        //
        // Considering this, we'll check the alignment. a had an alignment of 8
        // and since b's row alignment is also 8, no padding is necessary.
        //
        // Nothing fancy here, just dump the bytes.
        for row in self.b.iter() {
            for v in row.iter() {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
        }

        // c
        // c, as we'll see, throws a very small wrench into things.
        bytes.extend_from_slice(&self.c.to_le_bytes());

        // Now, before c, we had an item with a size of 32 slot perfectly with
        // an item with a size of 8, making a total size of 40. And with an
        // alignment of 8, this struct could have been done. However, with c in
        // the mix, adding 4 means the structure's size is no longer a multiple
        // of its alignment, 8. We need to pad 4 bytes to bring its size up to 48.
        //
        // The main point of this inner structure (besides demonstrating the thing
        // with matrices,) is to show struct size padding in action.
        bytes.resize(bytes.len() + 4, 0);

        bytes
    }
}