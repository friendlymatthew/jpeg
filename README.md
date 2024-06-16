# jpeg-decoder

<br />

<p align="center">
    <img src="mike.jpg" alt="Original Mike" width="320"/>
    <img src="gray_mike.png" alt="Grayscale Mike" width="320"/>
</p>

## Coding Process

The decoder uses baseline sequential as its coding process. Other DCT-based coding process are soon to come!
See `CODING_PROCESSES.md`.

## Vectorization

`jpeg-decoder` makes use of portable simd during various steps of the decoding process - like scanning for markers,
level changing, grayscaling, and many more.

## Resources

[ITU-T.81 Specification](https://www.w3.org/Graphics/JPEG/itu-t81.pdf)
