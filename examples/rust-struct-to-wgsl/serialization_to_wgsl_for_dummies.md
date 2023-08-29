# Serialization to WGSL for Dummies

## About this document

This document has two major purposes:

1. To enumerate and explain the rules which WGSL uses to lay out values in memory (specifically structures).
2. Provide a sort of walkthrough/tutorial like guide to, given a desired structure in WGSL, fill a buffer such that it can be bound to that input storage/uniform variable.

(TODO: Add brief summary of document with toc?)

Note that references to the specification are provided for those trying to follow along using the spec. If you're looking for a chance to read the spec with training wheels on, now is the time. The spec references and technical speak will go away when we get to concrete examples and actually formatting buffers although concepts from the theory section on how WGSL lays out values in memory.

## How WGSL Lays Out Values in Memory (AKA Section 13.4 of the Spec, Human Translation)

If you want to lay out data in a buffer such that WGSL correctly interprets it as a certain structure, it's important to understand how WGSL interprets values in the first place. How WGSL lays out values in memory is layed out in [section 13.4 of the WGSL specification](https://www.w3.org/TR/2023/WD-WGSL-20230802/#memory-layouts) however, the section is long and it's very easy to miss important things if you don't read it cover to cover and are able to successfully wade through all the technical speak. Note that this guide is also guilty of placing important information in non-obvious places because it does assume you read though at least most of it like a guide and takes the position that it's better to give information piecemeal as it comes up rather than overwhelming the reader all at once. If you want to make sure you're not missing anything, see the bullet list above.

Let's break it down and first, go through each basic type and talk about how it is stored in memory (and thus in what format WGSL expects it in a buffer). Note that this will get a little technical so if you're not interested in the nitty-gritty and are just interested in learning how to format a buffer for them, skip to the next section where we learn how to do just that.

### Integer Values

We start at [13.4.4, "Internal Layout of Values"](https://www.w3.org/TR/2023/WD-WGSL-20230802/#internal-value-layout) where the internal layouts of the basic numerical types are being described. According to the specification, integer values are layed out in ascending [bit order](https://en.wikipedia.org/wiki/Bit_numbering) (aka "little endian" byte order) with the most significant bit being used as the sign bit for signed integer values.

### Floating Point Values

Directly below integers, for those following along in the spec, is the specification for floats. `f32`'s are stored in the IEEE-754 floating point binary32 format. The official IEEE specification has a pricy paywall but "[you can get all the important stuff on the Wikipedia page instead for free](https://en.wikipedia.org/wiki/IEEE_754)". WGSL also supports `f16`s which use IEEE-754 binary16 format but these are highly unusual and you need to do special things to use them. Once again, little endian byte order for floats as well.

### Atomic Values

Continuing through 13.4.4, atomic values are by far the easiest. Atomic values under the hood are really just a wrapper around a `i32` or `u32` that only allows atomic operations on the underlying data.

### Vectors

`vec`'s are actually easier than arrays since they, unlike arrays, by virtue of the limitation of what types can be a vector, are always perfectly packed. By that is meant that the next element always begins where the last ended, which, as we'll see later, is not always true for arrays.

If we have a `vec3<f32>`, it's internal layout would look like this where the composing elements are identified by their index:

+---+---+---+---+---+---+---+---+---+---+---+---+
| 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 2 | 2 | 2 | 2 |
+---+---+---+---+---+---+---+---+---+---+---+---+

We'll get familiar with the `vec3<f32>` as it's also a great example of when alignment comes into play for later.

As for how to convert this to WGSL-compatible bytes, simply follow the hint of the diagram above and concatenate the bytes of each element together.

### Matrices

If you've worked with WGSL matrices, you may know that indexing them [gives a `vec` representing a column](https://www.w3.org/TR/2023/WD-WGSL-20230802/#matrix-access-expr) rather than a row as you might expect. This is because WGSL matrices are [column-major](https://en.wikipedia.org/wiki/Row-_and_column-major_order) both in interface and memory layout. Essentially, internally, matrices `matCxR<T>` are essentially `array<vecR<T>, C>`'s and should be treated as such for the purposes of creating one in a buffer.

A Rust representation of, say, `mat4x3<f32>` might look like `[[f32; 3]; 4]` but it's important to note that we can't just flatten this structure and fill a buffer with the string of `f32`'s due to alignment. Matrices are the first type in this guide where we run into padding when the row count is odd. This is due to that internal representation of an array and how array elements must be aligned. For more information on this, see the summary of the internal representation of arrays below.

### Arrays

Arrays is WGSL are relatively simple and straightforward but with a twist: the values contained are layed out sequentially but if you aren't used to working with this sort of stuff, it may confuse you to hear that elements in arrays must be aligned.

WGSL guarantees that values are properly aligned and if necessary, leaves gaps in memory to place the next array element or structure member starting at an offset that is a multiple of its alignment. This, as we'll see in the section about actually formatting buffers ([How to Create a WGSL-friendly Byte Buffer for Any Type](#how-to-create-a-wgsl-friendly-byte-buffer-for-any-type)), means that we will need to predict this and add padding accordingly to align the values either for reading from or writing to WGSL buffers.

What is the alignment of a value in WGSL? That's a question best answered in this guide's companion piece, [Alignment in WGSL](./alignment_in_wgsl.md). For now, understand that each type has a certain alignment that it likes to be aligned to in terms of offset from the beginning of a buffer. This is to aid performance although the in-depths of how this works are far beyond the scope of this guide.

For the nerds (everyone else can skip to [Structure Types](#structure-types)), we know about this because according to 14.4.4, "When a value of array type A is placed at byte offset k of a host-shared memory buffer, then: Element i of the array is placed at byte offset `k + i × StrideOf(A)`".

`StrideOf` represents the [array stride](https://en.wikipedia.org/wiki/Stride_of_an_array), meaning the number of memory locations (bytes) between the starts of the elements of the array. `StrideOf(array<E>)` is defined in [13.4, "Memory Layout"](https://www.w3.org/TR/2023/WD-WGSL-20230802/#memory-layouts) as `roundUp(AlignOf(E), SizeOf(E))`, meaning the size of the element type, rounded up to the nearest multiple of that type's alignment. Again, see Alignment is WGSL as mentioned above for details on getting the alignment of a type.

Because the size of the type might not neatly fit into the type's alignment, such as `vec3`s whose alignment is larger than their size, the stride of the array may wind up skipping some bytes between elements, thus, padding.

### Structure Types

Structures, are, as expected, by far the most complicated. The section on them in 13.4.4 basically just effectively immediately refers us to [13.4.2, "Structure Member Layout"](https://www.w3.org/TR/2023/WD-WGSL-20230802/#structure-member-layout) which is where most of the relevant information is placed. What makes them extra complicated to discuss is that structure members can have attributes that can impact the way they are represented in memory. Attributes that impact the layout of a structure in memory are known as "layout attributes" and they are as follows:

- [`size`](https://www.w3.org/TR/2023/WD-WGSL-20230802/#attribute-size)
- [`align`](https://www.w3.org/TR/2023/WD-WGSL-20230802/#attribute-align)

As the spec says, "the members are arranged tightly, in order, without overlap, while satisfying member alignment requirements". Let's break that down as a set of concrete, easy to follow rules for placing members. Each member can be found at the earliest memory offset from the start of the parent structure that satisfies the following conditions:

1. The member must be placed after the member before it; the members are ordered in memory the way they were in the structure definition. Before you ask, yes, due to alignment, an implication of this is that the order you place members in the definition can change the overall size of the structure due to extra inter-member padding.
2. The memory taken up by the member must not overlap with the memory of any other member. This should probably be pretty self-evident.
3. The member must be aligned. Remember that the alignment of the member is not necessarily the alignment of its type due to the `align` attribute. Again, see Alignment in Wgsl.

For those into the technical nitty-gritty yet too lazy to read the spec, the offset of a member (`i`) from the beginning of its parent struct (`S`) is formally notated as `OffsetOfMember(S, i)` and `OffsetOfMember` has the following definition:

1. `OffsetOfMember(S, 1) = 0`
2. `OffsetOfMember(S, i) = roundUp(AlignOfMember(S, i), OffsetOfMember(S, i-1) + SizeOfMember(S, i-1))` where `i > 1`

Again, note that "AlignOfMember" and "SizeOfMember" are used and not `AlignOf(M)` and `SizeOf(M)` due to the fact that member alignments and sizes may be different than those of their types.

If all this theoretical speak is sliding off your brain or you're starting to get confused, it's likely some concrete examples will help in the relevant part of [How to Create a WGSL-Friendly Byte Buffer for Any Type](#how-to-create-a-wgsl-friendly-byte-buffer-for-any-type) (TODO: Link the actual section).

## How to Create a WGSL-Friendly Byte Buffer for Any Type

## The Uniform Memory Space (and What's Special About it)