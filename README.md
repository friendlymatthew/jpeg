# jpeg
jpeg is a JFIF codec that implements .jpeg algorithms.

<br />

<p align="center">
    <img src="mike.jpg" alt="Original Mike" width="300"/>
    <img src="gray_mike.png" alt="Grayscale Mike" width="300"/>
</p>



# Specs
## Coding process 
- [x] baseline sequential 
- [ ] extended DCT-based decoding 
- [ ] lossless 

## Modes of operation
- [x] sequential DCT-based
- [ ] progressive DCT-based
- [ ] lossless
- [ ] hierarchical

## Entry coding procedure
- [x] Huffman tables
- [ ] Arithmetic coding conditioning tables


## SIMD
jpeg makes use of `std::simd` throughout various steps. For instance, JFIF marker detection and grayscaling use SIMD operations.


## ITU-T.81 Specification
https://www.w3.org/Graphics/JPEG/itu-t81.pdf